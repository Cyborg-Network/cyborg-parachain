use crate::{mock::*, Error, NextTaskId, PaymentPurpose, TaskAllocations, Tasks};
pub use cyborg_primitives::miner::*;
use cyborg_primitives::task::{OnnxTask, OpenInferenceTask, TaskSubmissionData};
use frame_support::{assert_noop, assert_ok};

use cyborg_primitives::payment::PaymentMode;
pub use cyborg_primitives::task::TaskStatusType;
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo},
	traits::{fungible::Mutate, OnInitialize},
	BoundedVec,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{traits::Get, TokenError};

use sp_std::convert::TryFrom;

fn register_miner(
	account: u64,
	miner_type: MinerType,
	domain_str: &str,
	miner_id: Vec<u8>,
) -> Result<(PostDispatchInfo, MinerId), DispatchErrorWithPostInfo> {
	// Clear previous events to avoid contamination
	frame_system::Pallet::<Test>::reset_events();

	let result = EdgeConnectModule::register_miner(
		RuntimeOrigin::signed(account),
		miner_type.clone(),
		miner_id,
		BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap(),
		590000,
		120000,
		10000000,
		10000000,
		12,
	);

	if result.is_ok() {
		// Now we know the only events are from this registration
		let system_events = frame_system::Pallet::<Test>::events();
		let bounded_miner_id = system_events
			.iter()
			.find_map(|event_record| {
				if let RuntimeEvent::EdgeConnectModule(
					pallet_edge_connect::Event::MinerRegistered { miner, .. },
				) = &event_record.event
				{
					Some(miner.1.clone())
				} else {
					None
				}
			})
			.expect("MinerRegistered event should be emitted");

		let _ = EdgeConnectModule::update_oracle_status(
			RuntimeOrigin::signed(account),
			account,
			bounded_miner_id.clone(),
			miner_type.clone(),
			true,
		);

		let _ = EdgeConnectModule::update_operational_status(
			RuntimeOrigin::signed(account),
			miner_type,
			bounded_miner_id.clone(),
			OperationalStatus::Available,
		);

		Ok((result.unwrap(), bounded_miner_id))
	} else {
		Err(result.err().unwrap())
	}
}

fn setup_treasury_account() {
	let account = pallet_payment::Pallet::<Test>::pallet_account_id();

	let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();

	let _ = Balances::mint_into(&account, existential_deposit).unwrap();
}

fn setup_user_with_active_payment(account: u64, mode: PaymentMode) {
	let rate = match mode {
		PaymentMode::OnDemand => <Test as pallet_payment::Config>::OnDemandRate::get(),
		PaymentMode::Subscription => <Test as pallet_payment::Config>::SubscriptionRate::get(),
	};

	let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();
	let required_balance = rate + existential_deposit;

	// Get current balance and calculate how much to mint
	let current_balance = Balances::free_balance(&account);
	let mint_amount =
		if current_balance < required_balance { required_balance - current_balance } else { 0 };

	// Mint additional balance if needed
	if mint_amount > 0 {
		let _ = Balances::mint_into(&account, mint_amount).unwrap();
	}
}

fn setup_gatekeeper() {
	TaskManagementModule::set_gatekeeper(RuntimeOrigin::root(), 1).unwrap();
}

#[test]
fn task_initialization_works() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let user = 1;
		let executor = 2;

		// Register a miner
		let miner_id =
			register_miner(executor, MinerType::Edge, "test.miner.0", b"test-miner-id-0".to_vec())
				.unwrap()
				.1;

		// Register a second miner
		let miner2_id =
			register_miner(executor, MinerType::Edge, "test.miner.1", b"test-miner-id-1".to_vec())
				.unwrap()
				.1;

		// Setup user with on-demand payment
		// current_balance + (on_demand_rate + existential_deposit)
		setup_user_with_active_payment(user, PaymentMode::OnDemand);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Schedule first task - should succeed
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Try to schedule second task with same miner - should fail
		assert_noop!(
			TaskManagementModule::initialize(
				RuntimeOrigin::signed(user),
				task_kind.clone(),
				miner_id.clone(),
				PaymentMode::OnDemand,
			),
			pallet_edge_connect::Error::<Test>::PendingTask
		);

		// Fails on user low balance, user must reserve payment for task execution
		assert_noop!(
			TaskManagementModule::initialize(
				RuntimeOrigin::signed(user),
				task_kind.clone(),
				miner2_id.clone(),
				PaymentMode::Subscription,
			),
			TokenError::NotExpendable
		);

		// Setup user with subscription payment
		// current_balance + (subscription_rate + existential_deposit)
		setup_user_with_active_payment(user, PaymentMode::Subscription);

		// User can submit similar task to a different miner
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner2_id.clone(),
			PaymentMode::Subscription,
		));

		let task2_id = NextTaskId::<Test>::get() - 1;

		// Confirm reception of first task to move it to running state
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id,
		));

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task2_id
		));

		// Verify both tasks exists for this user
		let user_tasks: Vec<_> = Tasks::<Test>::iter()
			.filter(|(_, task_info)| task_info.task_owner == user)
			.collect();
		assert_eq!(user_tasks.len(), 2);
	});
}

#[test]
fn tasks_cancellation_works() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let user = 1;
		let executor = 2;

		// Register multiple miners
		let miner1_id =
			register_miner(executor, MinerType::Edge, "miner1.test", b"miner-id-11111".to_vec())
				.unwrap()
				.1;

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let on_demand_rate = <Test as pallet_payment::Config>::OnDemandRate::get();
		let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();
		let required_balance = on_demand_rate + existential_deposit;

		// Setup user with  balance for a single payment
		// current_balance + (on_demand_rate + existential_deposit)
		setup_user_with_active_payment(user, PaymentMode::OnDemand);

		// User free balance before first task assignment
		let ub_before = Balances::free_balance(&user);
		assert_eq!(ub_before, required_balance);

		// Schedule first task
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner1_id.clone(),
			PaymentMode::OnDemand,
		));

		// TODO: Check all storage creation and state.

		// User free balance after first task assignment
		let ub_after = Balances::free_balance(&user);
		assert_eq!(ub_after, required_balance - on_demand_rate);

		let task_id = NextTaskId::<Test>::get() - 1;

		// Cancel first task to free up payment
		assert_ok!(TaskManagementModule::cancel_task(RuntimeOrigin::signed(user), task_id,));

		// TODO: Check all storage cleanup and state.

		let ub_cancel = Balances::free_balance(&user);
		assert_eq!(ub_cancel, required_balance);

		setup_user_with_active_payment(user, PaymentMode::Subscription);

		// Now should be able to schedule new task
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner1_id.clone(),
			PaymentMode::Subscription,
		));

		let task2_id = NextTaskId::<Test>::get() - 1;
		assert_ne!(task_id, task2_id);

		// Confirm one task exist (one cancelled and storage cleared, one active)
		let user_tasks: Vec<_> = Tasks::<Test>::iter()
			.filter(|(_, task_info)| task_info.task_owner == user)
			.collect();
		assert_eq!(user_tasks.len(), 1);
	});
}

#[test]
fn task_cleanup_after_payment_expiration() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let user = 1;
		let executor = 2;

		let miner_id =
			register_miner(executor, MinerType::Edge, "expire.miner", b"expire-miner-id".to_vec())
				.unwrap()
				.1;

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Setup user with on-demand payment
		setup_user_with_active_payment(user, PaymentMode::OnDemand);

		// Schedule and confirm task
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id,
		));

		// Verify task is running and payment is active
		let task_before = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task_before.task_status, TaskStatusType::Running);
		assert!(pallet_payment::Pallet::<Test>::has_active_payment(
			&user,
			PaymentPurpose::TaskExecution(task_id)
		));

		// Simulate payment expiration
		let on_demand_period: BlockNumberFor<Test> =
			<Test as pallet_payment::Config>::OnDemandPeriod::get();
		System::set_block_number(on_demand_period + 2);

		// Process payment expirations
		TaskManagementModule::on_initialize(System::block_number());

		// Verify task is stopped and cleaned up
		assert!(!Tasks::<Test>::contains_key(task_id));
		assert!(!TaskAllocations::<Test>::contains_key(task_id));

		// Verify miner is vacated
		let miner_info = pallet_edge_connect::EdgeMiners::<Test>::get(miner_id.clone()).unwrap();
		assert_eq!(miner_info.operational_status, OperationalStatus::Available);
		assert_eq!(miner_info.current_task, None);

		// Now user can schedule new task since old payment expired
		setup_user_with_active_payment(user, PaymentMode::OnDemand);

		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner_id.clone(),
			PaymentMode::OnDemand,
		));
	});
}

#[test]
fn task_termination_comprehensive() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let user = 1;
		let executor = 2;

		let miner_id = register_miner(
			executor,
			MinerType::Edge,
			"termination.miner",
			b"termination-miner-id".to_vec(),
		)
		.unwrap()
		.1;

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Setup user with sufficient balance
		let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();
		let on_demand_rate = <Test as pallet_payment::Config>::OnDemandRate::get();
		let required_balance = (on_demand_rate * 2) + existential_deposit;

		let current_balance = Balances::free_balance(&user);
		if current_balance < required_balance {
			let _ = Balances::mint_into(&user, required_balance - current_balance).unwrap();
		}

		// 1. Schedule and confirm task
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id,
		),);

		// Verify task is running
		let task_info = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task_info.task_status, TaskStatusType::Running);
		assert!(pallet_payment::Pallet::<Test>::has_active_payment(
			&user,
			PaymentPurpose::TaskExecution(task_id)
		));

		// 2. Test termination with minimal execution time
		System::set_block_number(2);

		// A subscription is required for task termination
		assert_noop!(
			TaskManagementModule::terminate(RuntimeOrigin::signed(user), task_id,),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn termination_with_subscription_payments() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let user = 1;
		let executor = 2;

		let miner_id = register_miner(
			executor,
			MinerType::Edge,
			"subscription.miner",
			b"subscription-miner-id".to_vec(),
		)
		.unwrap()
		.1;

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Setup user with subscription payment
		let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();
		let subscription_rate = <Test as pallet_payment::Config>::SubscriptionRate::get();
		let required_balance = subscription_rate + existential_deposit;

		let current_balance = Balances::free_balance(&user);
		if current_balance < required_balance {
			let _ = Balances::mint_into(&user, required_balance - current_balance).unwrap();
		}

		// Schedule with subscription payment
		assert_ok!(TaskManagementModule::initialize(
			RuntimeOrigin::signed(user),
			task_kind.clone(),
			miner_id.clone(),
			PaymentMode::Subscription,
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id,
		));

		// Execute for some time
		System::set_block_number(100);

		let balance_before_termination = Balances::free_balance(&user);

		assert_ok!(TaskManagementModule::terminate(RuntimeOrigin::signed(user), task_id,));

		let balance_after_termination = Balances::free_balance(&user);
		assert!(
			balance_after_termination > balance_before_termination,
			"Should receive cashback for subscription termination"
		);

		// Verify cleanup
		assert!(!Tasks::<Test>::contains_key(task_id));
		assert!(!pallet_payment::ActivePayments::<Test>::contains_key(
			&user,
			PaymentPurpose::TaskExecution(task_id)
		));
	});
}
