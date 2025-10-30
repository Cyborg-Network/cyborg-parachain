use crate::{mock::*, Error};
use crate::{
	ComputeAggregations, GatekeeperAccount, ModelHashes, NextTaskId, PendingTaskConfirmations,
	ResetReason, TaskAllocations, TaskAssignmentBlock, TaskStatus, Tasks,
};
pub use cyborg_primitives::miner::*;
pub use cyborg_primitives::task::NeuroZkTaskSubmissionDetails;
use cyborg_primitives::task::{
	AzureTask, OnnxTask, OpenInferenceTask, TaskId, TaskSubmissionData,
};
use frame_support::{assert_noop, assert_ok};
use sp_core::ConstU32;


pub use cyborg_primitives::miner::*;
use cyborg_primitives::payment::PaymentMode;

pub use cyborg_primitives::task::{TaskKind, TaskStatusType, NzkData};
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use frame_support::traits::fungible::Mutate;
use frame_support::traits::Currency;
use frame_support::traits::OnInitialize;
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::Get;
use sp_runtime::DispatchError;

use sp_runtime::DispatchResult;
use sp_std::convert::TryFrom;
use pallet_edge_connect::{CloudMiners, EdgeMiners};

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
        590000, 120000, 10000000, 10000000, 12,
    );

    if result.is_ok() {
        // Now we know the only events are from this registration
        let system_events = frame_system::Pallet::<Test>::events();
        let bounded_miner_id = system_events.iter()
            .find_map(|event_record| {
                if let RuntimeEvent::EdgeConnectModule(
                    pallet_edge_connect::Event::MinerRegistered { miner, .. }
                ) = &event_record.event {
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

fn setup_user_with_active_payment(account: u64, mode: PaymentMode) {
	let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();

	let rate = match mode {
		PaymentMode::OnDemand => <Test as pallet_payment::Config>::OnDemandRate::get(),
		PaymentMode::Subscription => <Test as pallet_payment::Config>::SubscriptionRate::get(),
	};

	let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();
	let required_balance = rate + existential_deposit * 2;

	// Get current balance and calculate how much to mint
	let current_balance = Balances::free_balance(&account);
	let mint_amount = if current_balance < required_balance {
		required_balance - current_balance
	} else {
		0
	};

	// Mint additional balance if needed
	if mint_amount > 0 {
		let _ = Balances::mint_into(&account, mint_amount).unwrap();
	}

	// Activate payment
	assert_ok!(PaymentModule::activate(
		RuntimeOrigin::signed(account),
		mode
	));
}

fn setup_service_provider_account() {
	let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();

	let provider = 99;

	Balances::make_free_balance_be(&provider, existential_deposit);

	pallet_payment::ServiceProviderAccount::<Test>::put(provider);
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
fn task_scheduler_works() {
    new_test_ext().execute_with(|| {
        setup_gatekeeper();
        setup_service_provider_account();
        System::set_block_number(1);

        let users = (1, 2, 3);
        let executor = 4;

        let (bounded_id_0, bounded_id_1, bounded_id_2, bounded_id_3) = (
            register_miner(executor, MinerType::Edge, "docker.miner",
                b"11111111-aaaa-bbbb-cccc-111111111111".to_vec()).unwrap().1,
            register_miner(executor, MinerType::Edge, "exec.miner",
                b"22222222-bbbb-cccc-dddd-222222222222".to_vec()).unwrap().1,
            register_miner(executor, MinerType::Edge, "robust.miner.on.demand",
                b"33333333-cccc-dddd-eeee-333333333333".to_vec()).unwrap().1,
            register_miner(executor, MinerType::Edge, "robust.miner.subscription",
                b"44444444-dddd-eeee-ffff-444444444444".to_vec()).unwrap().1,
        );

        // Verify miners are registered using the actual bounded_miner_id
        assert!(EdgeMiners::<Test>::contains_key(
            bounded_id_0.clone()
        ));
        assert!(EdgeMiners::<Test>::contains_key(
            bounded_id_1.clone()
        ));
        assert!(EdgeMiners::<Test>::contains_key(
            bounded_id_2.clone()
        ));
        assert!(EdgeMiners::<Test>::contains_key(
            bounded_id_3.clone()
        ));    

		let azure_task = AzureTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
		};

		let task_kind_neurozk = TaskSubmissionData::NeuroZK(NeuroZkTaskSubmissionDetails {
			location: azure_task.clone(),
			zk_input: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec())
				.unwrap(),
			zk_settings: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec())
				.unwrap(),
			zk_verifying_key: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			gatekeeper_pub: Some(BoundedVec::try_from([0u8; 32].to_vec()).unwrap()),
		});
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Provide payment for compute
		setup_user_with_active_payment(users.0, PaymentMode::OnDemand);
		setup_user_with_active_payment(users.1, PaymentMode::Subscription);
		setup_user_with_active_payment(users.2, PaymentMode::OnDemand);
		setup_user_with_active_payment(users.2, PaymentMode::Subscription);


        

        let miner_status = EdgeMiners::<Test>::get(bounded_id_0.clone());
        println!("Miner 0 status before scheduling: {:?}", miner_status);

        let miner_status = EdgeMiners::<Test>::get(bounded_id_1.clone());
        println!("Miner 1 status just before scheduling: {:?}", miner_status);

		// --------------------------------------------------
		// ✅ Schedule OpenInference Executable Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(users.0),
			task_kind_infer.clone(),
			executor,
			bounded_id_0.clone(),
			PaymentMode::OnDemand,
		));

		let task_id_0 = NextTaskId::<Test>::get() - 1;
		let task_info_0 = Tasks::<Test>::get(task_id_0).unwrap();
		assert_eq!(
			task_info_0.task_kind,
			TaskKind::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
				)
				.unwrap(),
				triton_config: None
			}))
		);

        let miner_status = EdgeMiners::<Test>::get(bounded_id_1.clone());
        println!("Miner 1 status before scheduling: {:?}", miner_status);

		// --------------------------------------------------
		// ✅ Schedule Neuro ZK Executable Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(users.1),
			task_kind_neurozk.clone(),
			executor,
			bounded_id_1.clone(), // TODO: Investigate to add a cloud miner insteaad.
			PaymentMode::Subscription,
		));

		let task_id_1 = NextTaskId::<Test>::get() - 1;
		let task_info_1 = Tasks::<Test>::get(task_id_1).unwrap();
		assert_eq!(
			task_info_1.task_kind,
			TaskKind::NeuroZK(NzkData {
				location: azure_task.clone(),
				zk_input: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec())
					.unwrap(),
				zk_settings: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
				)
				.unwrap(),
				zk_verifying_key: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
				)
				.unwrap(),
				zk_proof: None,
				last_proof_accepted: None
			})
		);

		// TODO: Multiple users should not access the same tasks simultaneously.
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(users.2),
			task_kind_infer.clone(),
			executor,
			bounded_id_2.clone(),
			PaymentMode::OnDemand
		));

		let task_id_2 = NextTaskId::<Test>::get() - 1;
		let task_info_2 = Tasks::<Test>::get(task_id_2).unwrap();
		assert_eq!(
			task_info_2.task_kind,
			TaskKind::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
				)
				.unwrap(),
				triton_config: None
			}))
		);

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(users.2),// TODO: Investigate second task of user.2 on cloud
            // miner.
			task_kind_neurozk.clone(),
			executor,
			bounded_id_3.clone(),
			PaymentMode::Subscription,
		));

		let task_id_3 = NextTaskId::<Test>::get() - 1;
		let task_info_3 = Tasks::<Test>::get(task_id_3).unwrap();
		assert_eq!(
			task_info_3.task_kind,
			TaskKind::NeuroZK(NzkData {
				location: azure_task,
				zk_input: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec())
					.unwrap(),
				zk_settings: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
				)
				.unwrap(),
				zk_verifying_key: BoundedVec::try_from(
					b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
				)
				.unwrap(),
				zk_proof: None,
				last_proof_accepted: None
			})
		);

		for task_id in &[task_id_0, task_id_1, task_id_2, task_id_3] {
			assert_ok!(TaskManagementModule::confirm_task_reception(
				RuntimeOrigin::signed(executor),
				*task_id,
			));
		}

		// Verify all tasks are running
		for task_id in &[task_id_0, task_id_1, task_id_2, task_id_3] {
			let task = Tasks::<Test>::get(*task_id).unwrap();
			assert_eq!(task.task_status, TaskStatusType::Running);
		}

		// Simulate payment expiration by advancing past on demand period
		let on_demand_period: cyborg_primitives::constants::BlockNumber =
			<Test as pallet_payment::Config>::OnDemandPeriod::get();
		System::set_block_number(on_demand_period + 2); // OnDemand past expiration

		TaskManagementModule::on_initialize(System::block_number());

		// Verify on demand payment tasks were stopped due to expired payment
		let task_after_0 = Tasks::<Test>::get(task_id_0).unwrap();
		let task_after_2 = Tasks::<Test>::get(task_id_2).unwrap();
		assert_eq!(task_after_0.task_status, TaskStatusType::Stopped);
		assert_eq!(task_after_2.task_status, TaskStatusType::Stopped);

		// Verify subscription tasks are still running
		let task_after_1 = Tasks::<Test>::get(task_id_1).unwrap();
		let task_after_3 = Tasks::<Test>::get(task_id_3).unwrap();
		assert_eq!(task_after_1.task_status, TaskStatusType::Running);
		assert_eq!(task_after_3.task_status, TaskStatusType::Running);

		// Verify miners were vacated for on-demand tasks
		let miner_info_docker =
			pallet_edge_connect::EdgeMiners::<Test>::get(bounded_id_0.clone()).unwrap();
		assert_eq!(miner_info_docker.operational_status, OperationalStatus::Available);
		assert_eq!(miner_info_docker.current_task, None);

		// Verify on demand payment miner for robust user is active &
		// subscription payment based miner is busy.
		// Note: A task per miner
		let miner_info_robust_on_demand =
			pallet_edge_connect::EdgeMiners::<Test>::get(bounded_id_2.clone()).unwrap();
		assert_eq!(miner_info_robust_on_demand.operational_status, OperationalStatus::Available);
		assert_eq!(miner_info_robust_on_demand.current_task, None);

		let miner_info_robust_subscription =
			pallet_edge_connect::EdgeMiners::<Test>::get(bounded_id_3.clone())
				.unwrap();
		assert_eq!(miner_info_robust_subscription.operational_status, OperationalStatus::Busy);
		assert_eq!(miner_info_robust_subscription.current_task, Some(task_id_3));

		// Simulate payment expiration by advancing past subscription period
		let subscription_period: cyborg_primitives::constants::BlockNumber =
			<Test as pallet_payment::Config>::SubscriptionPeriod::get();
		let grace_period: cyborg_primitives::constants::BlockNumber =
			<Test as pallet_payment::Config>::GracePeriod::get();
		System::set_block_number(subscription_period + grace_period + 2); // Subscription past expiration

		TaskManagementModule::on_initialize(System::block_number());

		// Verify subscription payment tasks were stopped due to expired payment
		let task_final_1 = Tasks::<Test>::get(task_id_1).unwrap();
		let task_final_3 = Tasks::<Test>::get(task_id_3).unwrap();
		assert_eq!(task_final_1.task_status, TaskStatusType::Stopped);
		assert_eq!(task_final_3.task_status, TaskStatusType::Stopped);

		// Verify all miners are now active
		for miner_id in &[
			bounded_id_0,
			bounded_id_1,
			bounded_id_2.clone(),
			bounded_id_3.clone(),
		] {
			let miner_info = pallet_edge_connect::EdgeMiners::<Test>::get(miner_id.clone()).unwrap();
			assert_eq!(miner_info.operational_status, OperationalStatus::Available);
			assert_eq!(miner_info.current_task, None);
		}

		// --------------------------------------------------
		// Additional Tests for User with Both Payments
		// --------------------------------------------------

		// Test: User with both payments tries to schedule task with expired on-demand but active subscription
		System::set_block_number(1); // Reset block number
		setup_user_with_active_payment(users.2, PaymentMode::Subscription); // Only subscription active

		// Should fail - on-demand payment not active
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(users.2),
				task_kind_infer.clone(),
				executor,
				bounded_id_2,
				PaymentMode::OnDemand,
			),
			Error::<Test>::InvalidPaymentMode
		);

		// Should succeed - subscription payment is active
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(users.2),
			task_kind_infer.clone(),
			executor,
			bounded_id_3,
			PaymentMode::Subscription,
		));
	})
}

/*
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

// #[test]
// fn test_register_and_retrieve_model_hash() {
//     new_test_ext().execute_with(|| {
//         // bring the trait into scope
//         use base64::Engine; 
//         use base64::engine::general_purpose::STANDARD;
//         use hex_literal::hex;
//         use sp_core::H256;

//         let gatekeeper = 1u64;
//         GatekeeperAccount::<Test>::put(gatekeeper);

//         let origin = RuntimeOrigin::signed(gatekeeper);

//         let model_id_vec =
//             hex!("79c3bc0974696a2ea9efd2f7bca19fdd630834bd0086f1b4a1c3db3dce3b2a51").to_vec();

//         let hash_b64 = "ecO8CXRpai6p79L3vKGf3WMINL0AhvG0ocPbPc47KlE=";

//         let hash_bytes = STANDARD.decode(hash_b64).expect("Valid base64");
//         assert_eq!(hash_bytes.len(), 32, "Hash must be 32 bytes");

//         let model_hash = H256::from_slice(&hash_bytes);

//         assert_ok!(TaskManagementModule::register_model_hash(
//             origin.clone(),
//             model_id_vec.clone(),
//             model_hash
//         ));

//         let mut model_id_fixed = [0u8; 32];
//         model_id_fixed.copy_from_slice(&model_id_vec);

// 		let stored_hash = ModelHashes::<Test>::get(model_id_fixed);
// 		assert_eq!(stored_hash, Some(model_hash));
// 	});
// }

#[test]
fn reset_task_should_work_for_stuck_assigned_task() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Verify task is in Assigned state
		let task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task.task_status, TaskStatusType::Assigned);

		// Verify miner is busy
		let miner = EdgeConnectModule::get_miner(&miner_id.clone(), &miner_type).unwrap();
		assert_eq!(miner.operational_status, OperationalStatus::Busy);
		assert_eq!(miner.current_task, Some(task_id));

		// Reset the stuck task as root using the helper
		assert_ok!(reset_task_as_root(
			task_id,
			miner_type.clone(),
			ResetReason::MinerUnresponsive
		));

		// Verify task is removed from storage
		assert!(Tasks::<Test>::get(task_id).is_none());
		assert!(TaskAllocations::<Test>::get(task_id).is_none());
		assert!(TaskStatus::<Test>::get(task_id).is_none());
		assert!(ComputeAggregations::<Test>::get(task_id).is_none());

		// Verify miner is reset to available
		let updated_miner = EdgeConnectModule::get_miner(&miner_id.clone(), &miner_type).unwrap();
		assert_eq!(
			updated_miner.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(updated_miner.current_task, None);

		// Check event emission
		System::assert_has_event(RuntimeEvent::TaskManagementModule(
			crate::Event::TaskManuallyReset {
				task_id,
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
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Schedule task and confirm reception
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id
		));

		// Verify task is in Running state
		let task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task.task_status, TaskStatusType::Running);

		// Reset the stuck running task using root
		assert_ok!(reset_task_as_root(
			task_id,
			miner_type.clone(),
			ResetReason::TaskTimeout
		));

		// Verify task is cleaned up
		assert!(Tasks::<Test>::get(task_id).is_none());
		assert!(TaskAllocations::<Test>::get(task_id).is_none());

		// Verify miner is reset
		let updated_miner = EdgeConnectModule::get_miner(&miner_id.clone(), &miner_type).unwrap();
		assert_eq!(
			updated_miner.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(updated_miner.current_task, None);
	});
}

#[test]
fn reset_task_should_work_for_stuck_stopped_task() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Schedule task and go through full lifecycle to Stopped state
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id
		));

		// Stop the task
		assert_ok!(TaskManagementModule::stop_task_and_vacate_miner(
			RuntimeOrigin::signed(alice),
			task_id
		));

		// Verify task is in Stopped state
		let task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task.task_status, TaskStatusType::Stopped);

		// Reset the stuck stopped task using root
		assert_ok!(reset_task_as_root(
			task_id,
			miner_type.clone(),
			ResetReason::ManualIntervention
		));

		// Verify task is cleaned up
		assert!(Tasks::<Test>::get(task_id).is_none());
		assert!(TaskAllocations::<Test>::get(task_id).is_none());

		// Verify miner is reset
		let updated_miner = EdgeConnectModule::get_miner(&miner_id.clone(), &miner_type).unwrap();
		assert_eq!(
			updated_miner.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(updated_miner.current_task, None);
	});
}

#[test]
fn reset_task_should_fail_for_non_root_caller() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner and create a task
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Non-root caller should fail
		assert_noop!(
			TaskManagementModule::reset_task(
				RuntimeOrigin::signed(alice),
				task_id,
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
			reset_task_as_root(
				nonexistent_task_id,
				miner_type,
				ResetReason::ManualIntervention
			),
			Error::<Test>::TaskNotFound
		);
	});
}

#[test]
fn reset_task_should_fail_for_non_resettable_states() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Create task and go through full lifecycle to Vacated state
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Complete the task lifecycle to reach Vacated state
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id
		));

		assert_ok!(TaskManagementModule::stop_task_and_vacate_miner(
			RuntimeOrigin::signed(alice),
			task_id
		));

		assert_ok!(TaskManagementModule::confirm_miner_vacation(
			RuntimeOrigin::signed(executor),
			task_id,
			miner_type.clone()
		));

		// Verify task is in Vacated state (non-resettable)
		let task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task.task_status, TaskStatusType::Vacated);

		assert_noop!(
			reset_task_as_root(task_id, miner_type, ResetReason::ManualIntervention),
			Error::<Test>::TaskNotResettable
		);
	});
}

#[test]
fn reset_task_should_handle_suspended_miner() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Suspend the miner
		assert_ok!(EdgeConnectModule::suspend_miner(
			RuntimeOrigin::root(),
			miner_id.clone(),
			miner_type.clone(),
			1000, // blocks
			SuspensionReason::TaskConfirmationTimeout
		));

		// Verify miner is suspended
		let miner = EdgeConnectModule::get_miner(&miner_id.clone(), &miner_type).unwrap();
		assert_eq!(miner.operational_status, OperationalStatus::Suspended);

		// Reset the task - should unsuspend the miner using root
		assert_ok!(reset_task_as_root(
			task_id,
			miner_type.clone(),
			ResetReason::SystemError
		));

		// Verify miner is no longer suspended and is available
		let updated_miner = EdgeConnectModule::get_miner(&miner_id.clone(), &miner_type).unwrap();
		assert_eq!(
			updated_miner.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(updated_miner.current_task, None);
	});
}

#[test]
fn reset_task_should_clean_up_pending_confirmations() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		// Register miner
		assert_ok!(register_miner(executor, miner_type.clone(), "exec.miner"));
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		let task_kind = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
			executor,
			miner_id.clone(),
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Verify task is in pending confirmations
		let assigned_block = TaskAssignmentBlock::<Test>::get(task_id).unwrap();
		let timeout_block = assigned_block.saturating_add(75);
		let pending_tasks = PendingTaskConfirmations::<Test>::get(timeout_block);

		// Debug output to help diagnose
		println!("Assigned block: {}", assigned_block);
		println!("Timeout block: {}", timeout_block);
		println!("Pending tasks at timeout block: {:?}", pending_tasks);

		assert!(
			pending_tasks.contains(&task_id),
			"Task should be in pending confirmations"
		);

		// Reset the task using root
		assert_ok!(reset_task_as_root(
			task_id,
			miner_type,
			ResetReason::ManualIntervention
		));

		// Verify task is removed from pending confirmations
		let pending_tasks_after = PendingTaskConfirmations::<Test>::get(timeout_block);
		assert!(
			!pending_tasks_after.contains(&task_id),
			"Task should be removed from pending confirmations"
		);
	});
}
*/
