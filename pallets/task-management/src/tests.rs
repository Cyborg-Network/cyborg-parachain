use crate::{mock::*, Error};
use crate::{
	ComputeAggregations, GatekeeperAccount, ModelHashes, NextTaskId, PendingTaskConfirmations,
	ResetReason, TaskAllocations, TaskAssignmentBlock, TaskStatus, Tasks,
};
pub use cyborg_primitives::miner::*;
pub use cyborg_primitives::task::NeuroZkTaskSubmissionDetails;
use cyborg_primitives::task::{AzureTask, OnnxTask, OpenInferenceTask, TaskId, TaskSubmissionData};
use frame_support::{assert_noop, assert_ok};
use sp_core::ConstU32;

pub use cyborg_primitives::task::{TaskKind, TaskStatusType};
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::DispatchError;
use sp_runtime::DispatchResult;
use sp_std::convert::TryFrom;

fn register_miner(
	account: u64,
	miner_type: MinerType,
	domain_str: &str,
) -> Result<(PostDispatchInfo, MinerId), DispatchErrorWithPostInfo> {
	// UUIDs for each miner
	// let miner_id  = b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec();

	let miner_id: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

  assert_ok!(EdgeConnectModule::add_account_authorized_for_registration(
      RuntimeOrigin::root(), 
      account
  ));

	let result = EdgeConnectModule::register_miner(
		RuntimeOrigin::signed(account),
		miner_type.clone(),
		miner_id.clone(),
		BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap(),
		590000,   // latitude
		120000,   // longitude
		10000000, // ram
		10000000, // storage
		12,       // cpu
	);

	if result.is_ok() {
		// Get the actual worker ID that was created
		let bounded_miner_id = pallet_edge_connect::AccountMiners::<Test>::get(account).unwrap();
		// Force set the oracle status to Online for testing
		let _ = EdgeConnectModule::update_oracle_status(
			RuntimeOrigin::signed(account),
			account,
			bounded_miner_id.clone(), // Use the actual worker ID
			miner_type.clone(),
			true, // online
		);

		// Set operational status to Available
		let _ = EdgeConnectModule::update_operational_status(
			RuntimeOrigin::signed(account),
			miner_type,
			bounded_miner_id.clone(), // Use the actual worker ID
			OperationalStatus::Available,
		);

		Ok((result.unwrap(), bounded_miner_id))
	} else {
		Err(result.err().unwrap())
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
fn it_works_for_task_scheduler() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> =
			b"ED-22222222-dddd-eeee-ffff-0987654321cd"
				.to_vec()
				.try_into()
				.unwrap();

		// Register workers first
		assert_ok!(register_miner(alice, MinerType::Edge, "docker.worker"));
		// assert_ok!(register_miner(executor, MinerType::Edge, "exec.worker"));

		// Verify workers are registered
		assert!(pallet_edge_connect::EdgeMiners::<Test>::contains_key(
			bounded_uuid_edge.clone()
		));
		// assert!(pallet_edge_connect::EdgeMiners::<Test>::contains_key(
		// 	bounded_uuid_edge.clone()
		// ));

		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// let miner_id_docker = 0;
		// let miner_id_exec = 1;

		let miner_id_docker: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// let miner_id_exec: BoundedVec<u8, ConstU32<64>> = 
		// 	b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		// Provide initial compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 50); // Increased for multiple tasks

		// // --------------------------------------------------
		// // ✅ Schedule OpenInference Executable Task (valid) - Use first worker
		// // --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer.clone(),
			miner_id_docker.clone(),
			Some(10)
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

		// let miner_id_zk: Vec<u8> = b"ED-22222222-dddd-eeee-ffff-0987654321aa".to_vec();
		let bounded_miner_id_zk: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321aa".to_vec().try_into().unwrap();
		let bob = 3;

    assert_ok!(EdgeConnectModule::add_account_authorized_for_registration(
        RuntimeOrigin::root(), 
        bob
    ));

		assert_ok!(EdgeConnectModule::register_miner(
		RuntimeOrigin::signed(bob),
		MinerType::Edge,
		bounded_miner_id_zk.clone(),
		BoundedVec::try_from("exec.worker".as_bytes().to_vec()).unwrap(),
		590000,   // latitude
		120000,   // longitude
		10000000, // ram
		10000000, // storage
		12,       // cpu
		));
	});
}

#[test]
fn it_works_for_miner_status_updates() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let miner_type = MinerType::Edge;

		assert_ok!(register_miner(executor, MinerType::Edge, "exec.miner"));

		let bounded_miner_id_exec: BoundedVec<u8, ConstU32<64>> =
			b"ED-22222222-dddd-eeee-ffff-0987654321cd"
				.to_vec()
				.try_into()
				.unwrap();

		// Verify miners are registered
		assert!(pallet_edge_connect::EdgeMiners::<Test>::contains_key(
			bounded_miner_id_exec
		));

		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// let miner_id_exec: BoundedVec<u8, ConstU32<64>> = BoundedVec::try_from(vec![0u8]).unwrap();
		let miner_id_exec: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();
		// Provide initial compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 30);

		// --------------------------------------------------
		// Schedule OpenInference Executable Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer.clone(),
			miner_id_exec.clone(),
			Some(10)
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

		// Make sure that task cannot be scheduled while miner is busy
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_infer.clone(),
				miner_id_exec.clone(),
				Some(10)
			),
			Error::<Test>::MinerIsBusy
		);

		// Confirm task reception
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			0
		));

		// Stop task and request miner vacation
		assert_ok!(TaskManagementModule::stop_task_and_vacate_miner(
			RuntimeOrigin::signed(alice),
			0
		));

		// Confirm miner has vacated
		assert_ok!(TaskManagementModule::confirm_miner_vacation(
			RuntimeOrigin::signed(executor),
			0,
			miner_type
		));

		// Schedule another task to the now free miner
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer.clone(),
			miner_id_exec.clone(),
			Some(10)
		));
	});
}

#[test]
fn it_fails_when_miner_not_registered() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let miner_owner = 2;
		let miner_id: BoundedVec<u8, ConstU32<64>> = BoundedVec::try_from(vec![99u8]).unwrap();

		// Register an Executable miner to ensure miners exist
		assert_ok!(register_miner(miner_owner, MinerType::Edge, "exec.miner"));

		let azure_task = AzureTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
		};

		// nzk_data only required for NeuroZK
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

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Attempt to schedule with non-existent miner ID
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_neurozk,
				miner_id.clone(),
				Some(1),
			),
			pallet_edge_connect::Error::<Test>::MinerDoesNotExist
		);
	});
}

#[test]
fn it_fails_when_no_miners_are_available() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let alice = 1;
		let miner_id: BoundedVec<u8, ConstU32<64>> = BoundedVec::try_from(vec![0u8]).unwrap();
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));
		// Provide an initial compute hours balance for Alice
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Ensure no miners exist
		assert!(pallet_edge_connect::CloudMiners::<Test>::iter()
			.next()
			.is_none());
		assert!(pallet_edge_connect::EdgeMiners::<Test>::iter()
			.next()
			.is_none());

		// Dispatch a signed extrinsic and expect an error because no miners are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_infer,
				miner_id.clone(),
				Some(10)
			),
			pallet_edge_connect::Error::<Test>::MinerDoesNotExist
		);
	});
}

#[test]
fn it_fails_when_no_computer_hours_available() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let alice = 1;

		let miner_owner = 2;
		let bounded_miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Register miner first
		assert_ok!(register_miner(miner_owner, MinerType::Edge, "miner.domain"));

		// Dispatch a signed extrinsic and expect an error because no miners are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_infer,
				bounded_miner_id.clone(),
				None
			),
			Error::<Test>::RequireComputeHoursDeposit
		);
	});
}

#[test]
fn confirm_task_reception_should_work_for_valid_assigned_miner() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let creator = 1;
		let executor = 2;
		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Register miner first
		assert_ok!(register_miner(executor, MinerType::Edge, "exec"));

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_kind_infer,
			miner_id.clone(),
			Some(10)
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id
		));

		// Task status should be updated to Running
		let task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task.task_status, TaskStatusType::Running);

		// Aggregation should have started
		let agg = ComputeAggregations::<Test>::get(task_id).unwrap();
		assert_eq!(agg.0, System::block_number());
		assert_eq!(agg.1, None);
	});
}

// #[test]
// fn confirm_task_reception_should_fail_for_wrong_executor() {
// 	new_test_ext().execute_with(|| {
// 		setup_gatekeeper();
// 		let creator = 1;
// 		let executor = 2;
// 		let intruder = 99;
// 		let miner_id: BoundedVec<u8, ConstU32<64>> = 
// 			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();


// 		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
// 			storage_location_identifier: BoundedVec::try_from(
// 				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
// 			)
// 			.unwrap(),
// 			triton_config: None,
// 		}));

// 		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
// 		assert_ok!(register_miner(executor, MinerType::Edge, "exec"));

// 		assert_ok!(TaskManagementModule::task_scheduler(
// 			RuntimeOrigin::signed(creator),
// 			task_kind_infer,
// 			executor,
// 			miner_id.clone(),
// 			Some(10)
// 		));

// 		let task_id = NextTaskId::<Test>::get() - 1;

// 		// Intruder tries to confirm task
// 		assert_noop!(
// 			TaskManagementModule::confirm_task_reception(RuntimeOrigin::signed(intruder), task_id),
// 			Error::<Test>::InvalidTaskOwner
// 		);
// 	});
// }

#[test]
fn confirm_task_reception_should_fail_if_already_running() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let creator = 1;
		let executor = 2;
		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);

		assert_ok!(register_miner(executor, MinerType::Edge, "exec"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_kind_infer,
			miner_id.clone(),
			Some(10)
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// First time (should work)
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id
		));

		// Second time (should fail - already running)
		assert_noop!(
			TaskManagementModule::confirm_task_reception(RuntimeOrigin::signed(executor), task_id),
			Error::<Test>::TaskReceptionAlreadyConfirmed
		);
	});
}

#[test]
fn it_works_for_confirm_miner_vacation() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));
		let miner_type = MinerType::Edge;

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Register an Executable miner
		assert_ok!(register_miner(alice, MinerType::Edge, "alice"));

		// 🔹 Submit task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			miner_id, // miner_id
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// 🔹 Confirm task reception
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id
		));

		// 🔹 Simulate that task was forcibly stopped
		Tasks::<Test>::mutate(task_id, |maybe_task| {
			if let Some(ref mut task) = maybe_task {
				task.task_status = TaskStatusType::Stopped;
			}
		});
		TaskStatus::<Test>::insert(task_id, TaskStatusType::Stopped);

		// 🔹 Call confirm_miner_vacation
		assert_ok!(TaskManagementModule::confirm_miner_vacation(
			RuntimeOrigin::signed(alice),
			task_id,
			miner_type
		));

		let updated_task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(updated_task.task_status, TaskStatusType::Vacated);
	});
}

// #[test]
// fn fails_if_not_assigned_miner_for_vacation() {
// 	new_test_ext().execute_with(|| {
// 		setup_gatekeeper();
// 		System::set_block_number(1);
// 		let alice = 1;
// 		let bob = 2;
// 		let miner_id: BoundedVec<u8, ConstU32<64>> = 
// 			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();
// 		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
// 			storage_location_identifier: BoundedVec::try_from(
// 				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
// 			)
// 			.unwrap(),
// 			triton_config: None,
// 		}));
// 		let miner_type = MinerType::Edge;

// 		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
// 		assert_ok!(register_miner(alice, MinerType::Edge, "alice"));

// 		assert_ok!(TaskManagementModule::task_scheduler(
// 			RuntimeOrigin::signed(alice),
// 			task_kind_infer,
// 			alice,
// 			miner_id,
// 			Some(5),
// 		));

// 		let task_id = NextTaskId::<Test>::get() - 1;

// 		assert_ok!(TaskManagementModule::confirm_task_reception(
// 			RuntimeOrigin::signed(alice),
// 			task_id
// 		));

// 		Tasks::<Test>::mutate(task_id, |maybe_task| {
// 			if let Some(ref mut task) = maybe_task {
// 				task.task_status = TaskStatusType::Stopped;
// 			}
// 		});
// 		TaskStatus::<Test>::insert(task_id, TaskStatusType::Stopped);

// 		// Bob is the task owner, but NOT the assigned miner
// 		assert_noop!(
// 			TaskManagementModule::confirm_miner_vacation(RuntimeOrigin::signed(bob), task_id, miner_type),
// 			Error::<Test>::NotAssignedMiner
// 		);
// 	});
// }

#[test]
fn fails_if_task_not_stopped() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));
		let miner_type = MinerType::Edge;

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);

		assert_ok!(register_miner(alice, MinerType::Edge, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			miner_id,
			Some(5),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Do not update status to Stopped → still Running
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id
		));

		//  Cannot confirm vacation unless status is Stopped
		assert_noop!(
			TaskManagementModule::confirm_miner_vacation(
				RuntimeOrigin::signed(alice),
				task_id,
				miner_type
			),
			Error::<Test>::InvalidTaskState
		);
	});
}

#[test]
fn it_works_for_stop_task_and_vacate_miner() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		// Provide compute hours and register miner
		pallet_payment::ComputeHours::<Test>::insert(alice, 40);

		assert_ok!(register_miner(alice, MinerType::Edge, "alice"));

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			miner_id,
			Some(10),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Confirm reception and set task to Running
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id,
		));
		Tasks::<Test>::mutate(task_id, |task| {
			if let Some(ref mut t) = task {
				t.task_status = TaskStatusType::Running;
			}
		});

		let now = System::block_number();
		ComputeAggregations::<Test>::insert(task_id, (now, None::<BlockNumberFor<Test>>));

		// Call extrinsic
		assert_ok!(TaskManagementModule::stop_task_and_vacate_miner(
			RuntimeOrigin::signed(alice),
			task_id,
		));

		let updated_task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(updated_task.task_status, TaskStatusType::Stopped);

		let agg = ComputeAggregations::<Test>::get(task_id).unwrap();
		assert_eq!(agg.0, 1);
		assert_eq!(agg.1, Some(System::block_number()));
	});
}

#[test]
fn fails_if_task_is_not_running() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(
				b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec(),
			)
			.unwrap(),
			triton_config: None,
		}));

		pallet_payment::ComputeHours::<Test>::insert(alice, 30);
		assert_ok!(register_miner(alice, MinerType::Edge, "alice"));

		// Schedule task and don't confirm reception (still Assigned)
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			miner_id,
			Some(15),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		//  Call stop while task is not Running
		assert_noop!(
			TaskManagementModule::stop_task_and_vacate_miner(RuntimeOrigin::signed(alice), task_id),
			Error::<Test>::InvalidTaskState
		);
	});
}

#[test]
fn fails_if_task_does_not_exist() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let task_id = 9999; // nonexistent task ID

		assert_noop!(
			TaskManagementModule::stop_task_and_vacate_miner(RuntimeOrigin::signed(1), task_id),
			Error::<Test>::TaskNotFound
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// Schedule task and confirm reception
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// Schedule task and go through full lifecycle to Stopped state
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// Create task and go through full lifecycle to Vacated state
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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

		let miner_id: BoundedVec<u8, ConstU32<64>> = b"ED-22222222-dddd-eeee-ffff-0987654321cd"
			.to_vec()
			.try_into()
			.unwrap();

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind,
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
