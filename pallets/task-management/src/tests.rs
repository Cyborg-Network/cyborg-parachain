use crate::{
	mock1::*, Error, GatekeeperAccount, ModelHashes, NextTaskId, PaymentPurpose,
	PendingTaskConfirmations, ResetReason, TaskAllocations, TaskAssignmentBlock, Tasks, PaymentDetailsOf, AssetIdOf
};
pub use cyborg_primitives::miner::*;
use cyborg_primitives::task::{
	AzureTask, NeuroZkTaskSubmissionDetails, OnnxTask, OpenInferenceTask, TaskId,
	TaskSubmissionData,
};

use cyborg_primitives::payment::{PaymentMode, PaymentRates};
pub use cyborg_primitives::task::{TaskKind, TaskStatusType};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchErrorWithPostInfo, DispatchResult, PostDispatchInfo},
	pallet_prelude::DispatchError,
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
	// Clear previous events
	frame_system::Pallet::<Test>::reset_events();

	assert_ok!(EdgeConnectModule::add_account_authorized_for_registration(
		RuntimeOrigin::root(),
		account
	));

	let mut miner_uuid_bytes = miner_id;

	let prefix = match miner_type {
		MinerType::Cloud => b"CL-",
		MinerType::Edge => b"ED-",
	};

	if !miner_uuid_bytes.starts_with(prefix) {
		let mut prefixed = prefix.to_vec();
		prefixed.extend_from_slice(&miner_uuid_bytes);
		miner_uuid_bytes = prefixed;
	}

	let bounded_miner_uuid: MinerId =
		BoundedVec::try_from(miner_uuid_bytes).expect("Out of bounds");

	let bounded_domain: Domain =
		BoundedVec::try_from(domain_str.as_bytes().to_vec()).expect("Out of bounds");

	let result = EdgeConnectModule::register_miner(
		RuntimeOrigin::signed(account),
		miner_type.clone(),
		bounded_miner_uuid,
		bounded_domain,
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
	let escrow = pallet_payment::Pallet::<Test>::account_id();

	let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();

	let _ = Balances::mint_into(&escrow, existential_deposit).unwrap();
}

fn setup_user_with_active_payment(account: u64, mode: PaymentMode, asset_id: AssetIdOf<Test>) {

    let rate = <Test as pallet_payment::Config>::Rate::get_rate(asset_id, mode);
	//let rate = match mode {
	//	PaymentMode::OnDemand => <Test as pallet_payment::Config>::OnDemandRate::get(),
	//	PaymentMode::Subscription => <Test as pallet_payment::Config>::SubscriptionRate::get(),
	//};

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

fn reset_task_as_root(
	task_id: TaskId,
	miner_type: MinerType,
	reason: ResetReason,
) -> DispatchResult {
	TaskManagementModule::reset_task(RuntimeOrigin::root(), task_id, miner_type, reason)
}

#[test]
fn scheduling() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let alice = 1;
		let bob = 2;
		let charlie = 3;
        let asset_id = 13;

		// Register alice as a miner
		let alice_miner_id =
			register_miner(alice, MinerType::Edge, "alice.miner", b"alice-miner-id".to_vec())
				.unwrap()
				.1;

		// Register a executor as a miner
		let charlie_miner_id = register_miner(
			charlie,
			MinerType::Edge,
			"executor.miner",
			b"executor-miner-id".to_vec(),
		)
		.unwrap()
		.1;

		// Register a second miner for executor
		let another_charlie_miner_id = register_miner(
			charlie,
			MinerType::Edge,
			"another.executor.miner",
			b"another-executor-miner-id".to_vec(),
		)
		.unwrap()
		.1;

		// Setup user with on-demand payment for bob
		setup_user_with_active_payment(bob, PaymentMode::OnDemand, asset_id);

		let task_kind_inference =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		// Bob schedules first task - should succeed
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(bob),
			task_kind_inference.clone(),
			charlie_miner_id.clone(),
			PaymentMode::OnDemand
            asset_id,
		));

		let bob_task_id = NextTaskId::<Test>::get() - 1;

		assert_eq!(bob_task_id, 0);

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(charlie),
			bob_task_id,
		));

		// Alice attempts to schedule first tasks - fails
		assert_noop!(
			TaskManagementModule::schedule(
				RuntimeOrigin::signed(alice),
				task_kind_inference.clone(),
				alice_miner_id.clone(),
				PaymentMode::Subscription,
			),
			TokenError::FundsUnavailable
		);

		// Bob schedules second task with same miner - should fail
		assert_noop!(
			TaskManagementModule::schedule(
				RuntimeOrigin::signed(bob),
				task_kind_inference.clone(),
				charlie_miner_id.clone(),
				PaymentMode::OnDemand,
			),
			pallet_edge_connect::Error::<Test>::Busy
		);

		// Mint for Bob
		setup_user_with_active_payment(bob, PaymentMode::OnDemand);

		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(bob),
			task_kind_inference.clone(),
			another_charlie_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let bob_second_task_id = NextTaskId::<Test>::get() - 1;

		assert_eq!(bob_second_task_id, 1);

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(charlie),
			bob_second_task_id,
		));

		// Setup user with subscription payment
		setup_user_with_active_payment(alice, PaymentMode::Subscription);

		// Alice schedules similar task as Bob to a their miner,
		// with subscription payment
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind_inference.clone(),
			alice_miner_id.clone(),
			PaymentMode::Subscription,
            asset_id,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		assert_eq!(alice_task_id, 2);

		// Confirm reception of first task to move it to running state
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			alice_task_id,
		));

		// Verify both tasks exists for this user
		let user_tasks: Vec<_> = Tasks::<Test>::iter()
			.filter(|(_, task_info)| task_info.task_owner == bob)
			.collect();
		assert_eq!(user_tasks.len(), 2);

		let user_tasks: Vec<_> = Tasks::<Test>::iter()
			.filter(|(_, task_info)| task_info.task_owner == alice)
			.collect();
		assert_eq!(user_tasks.len(), 1);
	});
}

#[test]
fn tasks_cancellation_works() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let alice = 1;
		let bob = 2;
        let asset_id = 14;

		// Register multiple miners
		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let rate = <Test as pallet_payment::Config>::Rate::get_rates(asset_id, PaymentMode::OnDemand);
		let existential_deposit = <Test as pallet_orml_tokens::Config>::ExistentialDeposit::get(14);
		let required_balance = rate + existential_deposit;

		// Setup user with  balance for a single payment
		// current_balance + (on_demand_rate + existential_deposit)
		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		// Alice free balance before scheduling first task
		let alice_balance_before_scheduling = Balances::free_balance(&alice);
		assert_eq!(alice_balance_before_scheduling, required_balance);

		// Schedule first task
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind.clone(),
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		// TODO: Check all storage creation and state.

		// User free balance after first task assignment
		let alice_balance_after_scheduling = Balances::free_balance(&alice);
		assert_eq!(alice_balance_after_scheduling, required_balance - rate);

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		// Cancel first task to free up payment
		assert_ok!(TaskManagementModule::cancel_task(RuntimeOrigin::signed(alice), alice_task_id,));

		// TODO: Check all storage cleanup and state.
		let alice_balance_after_cancellation = Balances::free_balance(&alice);
		assert_eq!(alice_balance_after_cancellation, required_balance);

		setup_user_with_active_payment(alice, PaymentMode::Subscription);

		// Now should be able to schedule new task with the same Id
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind.clone(),
			bob_miner_id.clone(),
			PaymentMode::Subscription,
		));

		let alice_second_task_id = NextTaskId::<Test>::get() - 1;
		assert_ne!(alice_task_id, alice_second_task_id);

		// Confirm one task exist (one cancelled and storage cleared, one active)
		let user_tasks: Vec<_> = Tasks::<Test>::iter()
			.filter(|(_, task_info)| task_info.task_owner == alice)
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

		let alice = 1;
		let bob = 2;
        let asset_id = 1;

		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
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
		setup_user_with_active_payment(alice, PaymentMode::OnDemand, asset_id);

		// Schedule and confirm task
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind.clone(),
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
            asset_id,
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			task_id,
		));

		// Verify task is running and payment is active
		let task_before = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task_before.task_status, TaskStatusType::Running);
		assert!(pallet_payment::Pallet::<Test>::has_active_payment(
			&alice,
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
		let miner_info =
			pallet_edge_connect::EdgeMiners::<Test>::get(bob_miner_id.clone()).unwrap();
		assert_eq!(miner_info.operational_status, OperationalStatus::Available);
		assert_eq!(miner_info.current_task, None);

		// Now user can schedule new task since old payment expired
		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind.clone(),
			bob_miner_id.clone(),
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

		let alice = 1;
		let bob = 2;
        let asset_id = 3;

		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
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
		setup_user_with_active_payment(alice, PaymentMode::OnDemand, asset_id);

		// 1. Schedule and confirm task
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind.clone(),
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
            assset_id,
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			task_id,
		),);

		// Verify task is running
		let task_info = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task_info.task_status, TaskStatusType::Running);
		assert!(pallet_payment::Pallet::<Test>::has_active_payment(
			&alice,
			PaymentPurpose::TaskExecution(task_id)
		));

		// 2. Test termination with minimal execution time
		System::set_block_number(2);

		// A subscription is required for task termination
		assert_noop!(
			TaskManagementModule::terminate(RuntimeOrigin::signed(alice), task_id,),
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

		let alice = 1;
		let bob = 2;

		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
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
		setup_user_with_active_payment(alice, PaymentMode::Subscription);

		// Schedule with subscription payment
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_kind.clone(),
			bob_miner_id.clone(),
			PaymentMode::Subscription,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			alice_task_id,
		));

		// Execute for some time
		System::set_block_number(100);

		let balance_before_termination = Balances::free_balance(&alice);

		// Invalid task id - should fail
		assert_noop!(
			TaskManagementModule::terminate(RuntimeOrigin::signed(alice), 9999),
			Error::<Test>::TaskNotFound
		);

		assert_ok!(TaskManagementModule::terminate(RuntimeOrigin::signed(alice), alice_task_id));

		let balance_after_termination = Balances::free_balance(&alice);
		assert!(
			balance_after_termination > balance_before_termination,
			"Should receive cashback for subscription termination"
		);

		// Verify cleanup
		assert!(!Tasks::<Test>::contains_key(alice_task_id));
		assert!(!pallet_payment::ActivePayments::<Test>::contains_key(
			&alice,
			PaymentPurpose::TaskExecution(alice_task_id)
		));
	});
}

#[test]
fn confirm_task_reception_should_work_for_valid_assigned_miner() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		// Bob register's miner
		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			alice_task_id,
		));

		// Task status should be updated to Running
		let alice_task_info = Tasks::<Test>::get(alice_task_id).unwrap();
		assert_eq!(alice_task_info.task_status, TaskStatusType::Running);
	});
}

#[test]
fn confirm_task_reception_should_fail_if_already_running() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		// First time (should work)
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			alice_task_id,
		));

		// Second time (should fail - already running)
		assert_noop!(
			TaskManagementModule::confirm_task_reception(RuntimeOrigin::signed(bob), alice_task_id),
			Error::<Test>::TaskReceptionAlreadyConfirmed
		);
	});
}

#[test]
fn test_register_model_hash_works() {
	new_test_ext().execute_with(|| {
		use hex_literal::hex;
		use sp_core::H256;

		let gatekeeper = 1;
		GatekeeperAccount::<Test>::put(gatekeeper.clone());

		let origin = RuntimeOrigin::signed(gatekeeper.clone());

		let model_id_hex = hex!("79c3bc0974696a2ea9efd2f7bca19fdd630834bd0086f1b4a1c3db3dce3b2a51");
		let model_id_vec = model_id_hex.to_vec();
		let model_hash = H256::repeat_byte(0x42);

		assert_ok!(TaskManagementModule::register_model_hash(
			origin,
			model_id_vec.clone(),
			model_hash
		));

		let mut fixed_id = [0u8; 32];
		fixed_id.copy_from_slice(&model_id_vec);
		assert_eq!(ModelHashes::<Test>::get(fixed_id), Some(model_hash));
	});
}

#[test]
fn reset_task_should_work_for_stuck_assigned_task() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);

		let alice = 1;
		let bob = 2;

		let bob_miner_id =
			register_miner(bob, MinerType::Edge, "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		// 1. Schedule task
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission.clone(),
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		// Verify task is in Assigned state
		let alice_task_info = Tasks::<Test>::get(alice_task_id).unwrap();
		assert_eq!(alice_task_info.task_status, TaskStatusType::Assigned);

		// Verify miner is assigned
		let bob_miner =
			EdgeConnectModule::get_miner(&bob_miner_id.clone(), &MinerType::Edge).unwrap();
		assert_eq!(bob_miner.operational_status, OperationalStatus::TaskAssigned);
		assert_eq!(bob_miner.current_task, Some(alice_task_id));

		// Reset the stuck task as root using the helper
		assert_ok!(reset_task_as_root(
			alice_task_id,
			MinerType::Edge,
			ResetReason::MinerUnresponsive
		));

		// Verify task is removed from storage
		assert!(Tasks::<Test>::get(alice_task_id).is_none());
		assert!(TaskAllocations::<Test>::get(alice_task_id).is_none());

		// Verify miner is reset to available
		let updated_miner =
			EdgeConnectModule::get_miner(&bob_miner_id.clone(), &MinerType::Edge).unwrap();
		assert_eq!(updated_miner.operational_status, OperationalStatus::Available);
		assert_eq!(updated_miner.current_task, None);

		// Check event emission
		System::assert_has_event(RuntimeEvent::TaskManagementModule(
			crate::Event::TaskManuallyReset {
				id: alice_task_id,
				reset_by: None, // root account
				previous_status: TaskStatusType::Assigned,
				reason: ResetReason::MinerUnresponsive,
			},
		));
	});
}

#[test]
fn reset_task_should_work_for_stuck_running_task() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		let bob_miner_id =
			register_miner(bob, miner_type.clone(), "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		// Schedule task and confirm reception
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			alice_task_id,
		));

		// Verify task is running
		let alice_task_info = Tasks::<Test>::get(alice_task_id).unwrap();
		assert_eq!(alice_task_info.task_status, TaskStatusType::Running);
		assert!(pallet_payment::Pallet::<Test>::has_active_payment(
			&alice,
			PaymentPurpose::TaskExecution(alice_task_id)
		));

		System::set_block_number(2);

		assert_noop!(
			TaskManagementModule::reset_task(
				RuntimeOrigin::signed(alice),
				alice_task_id,
				miner_type,
				ResetReason::ManualIntervention
			),
			DispatchError::BadOrigin
		);

		// Reset the stuck task as root using the helper
		assert_ok!(reset_task_as_root(
			alice_task_id,
			MinerType::Edge,
			ResetReason::MinerUnresponsive
		));
	});
}

#[test]
fn reset_task_should_fail_for_non_root_caller() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let miner_type = MinerType::Edge;

		// Register miner and create a task
		let bob_miner_id =
			register_miner(bob, miner_type.clone(), "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		// Non-root caller should fail
		assert_noop!(
			TaskManagementModule::reset_task(
				RuntimeOrigin::signed(alice),
				alice_task_id,
				miner_type,
				ResetReason::ManualIntervention
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn reset_task_should_fail_for_nonexistent_task() {
	new_test_ext().execute_with(|| {
		let nonexistent_task_id = 9999;
		let miner_type = MinerType::Edge;

		assert_noop!(
			reset_task_as_root(nonexistent_task_id, miner_type, ResetReason::ManualIntervention),
			Error::<Test>::TaskNotFound
		);
	});
}

#[test]
fn reset_task_should_fail_for_terminated_tasks() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		let bob_miner_id =
			register_miner(bob, miner_type.clone(), "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		setup_user_with_active_payment(alice, PaymentMode::Subscription);

		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id,
			PaymentMode::Subscription,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			alice_task_id
		));

		assert_ok!(TaskManagementModule::terminate(RuntimeOrigin::signed(alice), alice_task_id));

		assert_noop!(
			reset_task_as_root(alice_task_id, miner_type, ResetReason::ManualIntervention),
			Error::<Test>::TaskNotFound
		);
	});
}

#[test]
fn reset_task_should_handle_suspended_miner() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		let bob_miner_id =
			register_miner(bob, miner_type.clone(), "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		// Schedule task
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		// Suspend the miner
		assert_ok!(EdgeConnectModule::suspend_miner(
			RuntimeOrigin::root(),
			bob_miner_id.clone(),
			miner_type.clone(),
			1000, // blocks
			SuspensionReason::TaskConfirmationTimeout
		));

		// Verify miner is suspended
		let bob_miner = EdgeConnectModule::get_miner(&bob_miner_id.clone(), &miner_type).unwrap();
		assert_eq!(bob_miner.operational_status, OperationalStatus::Suspended);

		// Reset the task - should unsuspend the miner using root
		assert_ok!(reset_task_as_root(alice_task_id, miner_type.clone(), ResetReason::SystemError));

		// Verify miner is no longer suspended and is available
		let updated_miner =
			EdgeConnectModule::get_miner(&bob_miner_id.clone(), &miner_type).unwrap();
		assert_eq!(updated_miner.operational_status, OperationalStatus::Available);
		assert_eq!(updated_miner.current_task, None);
	});
}

#[test]
fn reset_task_should_clean_up_pending_confirmations() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		setup_treasury_account();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		let bob_miner_id =
			register_miner(bob, miner_type.clone(), "bob.miner", b"bob-miner-id".to_vec())
				.unwrap()
				.1;

		let task_inference_submission =
			TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
				)
				.unwrap(),
				triton_config: None,
			}));

		setup_user_with_active_payment(alice, PaymentMode::OnDemand);

		// Schedule task
		assert_ok!(TaskManagementModule::schedule(
			RuntimeOrigin::signed(alice),
			task_inference_submission,
			bob_miner_id.clone(),
			PaymentMode::OnDemand,
		));

		let alice_task_id = NextTaskId::<Test>::get() - 1;

		// Verify task is in pending confirmations
		let assigned_block = TaskAssignmentBlock::<Test>::get(alice_task_id).unwrap();
		let timeout_block = assigned_block.saturating_add(75);
		let pending_tasks = PendingTaskConfirmations::<Test>::get(timeout_block);

		// Debug output to help diagnose
		println!("Assigned block: {}", assigned_block);
		println!("Timeout block: {}", timeout_block);
		println!("Pending tasks at timeout block: {:?}", pending_tasks);

		assert!(pending_tasks.contains(&alice_task_id), "Task should be in pending confirmations");

		// Reset the task using root
		assert_ok!(reset_task_as_root(alice_task_id, miner_type, ResetReason::ManualIntervention));

		// Verify task is removed from pending confirmations
		let pending_tasks_after = PendingTaskConfirmations::<Test>::get(timeout_block);
		assert!(
			!pending_tasks_after.contains(&alice_task_id),
			"Task should be removed from pending confirmations"
		);
	});
}
