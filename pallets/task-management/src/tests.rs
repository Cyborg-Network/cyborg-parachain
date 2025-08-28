use crate::{mock::*, Error};
use crate::{ComputeAggregations, GatekeeperAccount, ModelHashes, NextTaskId, TaskStatus, Tasks};
use cyborg_primitives::task::{AzureTask, OnnxTask, OpenInferenceTask, TaskSubmissionData, NzkData};
pub use cyborg_primitives::task::NeuroZkTaskSubmissionDetails;
use frame_support::{assert_noop, assert_ok};

pub use cyborg_primitives::task::{TaskKind, TaskStatusType};
pub use cyborg_primitives::worker::*;
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::convert::TryFrom;

fn register_worker(
	account: u64,
	worker_type: WorkerType,
	domain_str: &str,
) -> Result<PostDispatchInfo, DispatchErrorWithPostInfo> {
	EdgeConnectModule::register_worker(
		RuntimeOrigin::signed(account),
		worker_type,
		BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap(),
		590000,   // latitude
		120000,   // longitude
		10000000, // ram
		10000000, // storage
		12,       // cpu
	)
}

fn setup_gatekeeper() {
	TaskManagementModule::set_gatekeeper(RuntimeOrigin::root(), 1).unwrap();
}

#[test]
fn it_works_for_task_scheduler() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;

		// Register workers first
		assert_ok!(register_worker(
			executor,
			WorkerType::Executable,
			"docker.worker"
		));
		assert_ok!(register_worker(
			executor,
			WorkerType::Executable,
			"exec.worker"
		));

		// Verify workers are registered
		assert!(pallet_edge_connect::ExecutableWorkers::<Test>::contains_key((executor, 0)));
		assert!(pallet_edge_connect::ExecutableWorkers::<Test>::contains_key((executor, 1)));

		let azure_task = AzureTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
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
		});
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		let worker_id_docker = 0;
		let worker_id_exec = 1;

		// Provide initial compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 30);

		// --------------------------------------------------
		// ✅ Schedule OpenInference Executable Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer.clone(),
			executor,
			worker_id_docker,
			Some(10)
		));

		let task_id_0 = NextTaskId::<Test>::get() - 1;
		let task_info_0 = Tasks::<Test>::get(task_id_0).unwrap();
		assert_eq!(
			task_info_0.task_kind, 
			TaskKind::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
				triton_config: None
			}))
		);

		// --------------------------------------------------
		// ✅ Schedule OpenInference Executable Task (valid)
		// --------------------------------------------------
		// assert_ok!(TaskManagementModule::task_scheduler(
		// 	RuntimeOrigin::signed(alice),
		// 	task_kind_infer.clone(),
		// 	task_data.clone(),
		// 	None,
		// 	executor,
		// 	worker_id_exec,
		// 	Some(10)
		// ));

		// let task_id_1 = NextTaskId::<Test>::get() - 1;
		// let task_info_1 = Tasks::<Test>::get(task_id_1).unwrap();
		// assert_eq!(task_info_1.task_kind, TaskKind::OpenInference);
		// assert_eq!(task_info_1.zk_files_cid, None);

		// --------------------------------------------------
		// ✅ Schedule NeuroZK Executable Task (valid with zk_files)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_neurozk,
			executor,
			worker_id_exec,
			Some(10)
		));

		let task_id_2 = NextTaskId::<Test>::get() - 1;
		let task_info_2 = Tasks::<Test>::get(task_id_2).unwrap();
		assert_eq!(
			task_info_2.task_kind, 
			TaskKind::NeuroZK(NzkData {
				location: azure_task,
				zk_input: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
				zk_settings: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
				zk_verifying_key: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
				zk_proof: None,
				last_proof_accepted: None
			})
		);
	});
}

#[test]
fn it_works_for_miner_status_updates() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;

		assert_ok!(register_worker(
			executor,
			WorkerType::Executable,
			"exec.worker"
		));

		// Verify workers are registered
		assert!(pallet_edge_connect::ExecutableWorkers::<Test>::contains_key((executor, 0)));

		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		let worker_id_exec = 0;

		// Provide initial compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 30);

		// --------------------------------------------------
		// ✅ Schedule OpenInference Executable Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer.clone(),
			executor,
			worker_id_exec,
			Some(10)
		));

		let task_id_0 = NextTaskId::<Test>::get() - 1;
		let task_info_0 = Tasks::<Test>::get(task_id_0).unwrap();
		assert_eq!(
			task_info_0.task_kind, 
			TaskKind::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
				storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
				triton_config: None
			}))
		);

		// Make sure that task cannot be scheduled while miner is busy
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_infer.clone(),
				executor,
				worker_id_exec,
				Some(10)
			),
			pallet_edge_connect::Error::<Test>::MinerIsBusy
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
			0
		));

		// Schedule another task to the now free miner
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),	
			task_kind_infer.clone(),
			executor,
			worker_id_exec,
			Some(10)
		));
	});
}

#[test]
fn it_fails_when_worker_not_registered() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let worker_owner = 2;
		let worker_id = 99;

		// Register an Executable worker to ensure workers exist
		assert_ok!(register_worker(
			worker_owner,
			WorkerType::Executable,
			"exec.worker"
		));

		let azure_task = AzureTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
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
		});

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Attempt to schedule with non-existent worker ID
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_neurozk,
				worker_owner,
				worker_id,
				Some(1),
			),
			pallet_edge_connect::Error::<Test>::WorkerDoesNotExist
		);
	});
}

#[test]
fn it_fails_when_no_workers_are_available() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let alice = 1;
		let worker_owner = 2;
		let worker_id = 0;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));
		// Provide an initial compute hours balance for Alice
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Ensure no workers exist
		assert!(pallet_edge_connect::WorkerClusters::<Test>::iter()
			.next()
			.is_none());
		assert!(pallet_edge_connect::ExecutableWorkers::<Test>::iter()
			.next()
			.is_none());

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_infer,
				worker_owner,
				worker_id,
				Some(10)
			),
			pallet_edge_connect::Error::<Test>::WorkerDoesNotExist
		);
	});
}

#[test]
fn it_fails_when_no_computer_hours_available() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let alice = 1;

		let worker_owner = 2;
		let worker_id = 0;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		// Register worker first
		assert_ok!(register_worker(
			worker_owner,
			WorkerType::Executable,
			"worker.domain"
		));

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_kind_infer,
				worker_owner,
				worker_id,
				None
			),
			Error::<Test>::RequireComputeHoursDeposit
		);
	});
}

#[test]
fn confirm_task_reception_should_work_for_valid_assigned_worker() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let creator = 1;
		let executor = 2;
		let worker_id = 0;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		// Register worker first
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_kind_infer,
			executor,
			worker_id,
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

#[test]
fn confirm_task_reception_should_fail_for_wrong_executor() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let creator = 1;
		let executor = 2;
		let intruder = 99;
		let worker_id = 0;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_kind_infer,
			executor,
			worker_id,
			Some(10)
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Intruder tries to confirm task
		assert_noop!(
			TaskManagementModule::confirm_task_reception(RuntimeOrigin::signed(intruder), task_id),
			Error::<Test>::InvalidTaskOwner
		);
	});
}

#[test]
fn confirm_task_reception_should_fail_if_already_running() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		let creator = 1;
		let executor = 2;
		let worker_id = 0;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_kind_infer,
			executor,
			worker_id,
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
			Error::<Test>::RequireAssignedTask
		);
	});
}

#[test]
fn it_works_for_confirm_miner_vacation() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));


		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Register an Executable worker
		assert_ok!(register_worker(alice, WorkerType::Executable, "alice"));

		// 🔹 Submit task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			alice,
			0, // worker_id
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
			task_id
		));

		let updated_task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(updated_task.task_status, TaskStatusType::Vacated);
	});
}

#[test]
fn fails_if_not_assigned_miner_for_vacation() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
		assert_ok!(register_worker(alice, WorkerType::Executable, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			alice,
			0,
			Some(5),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id
		));

		Tasks::<Test>::mutate(task_id, |maybe_task| {
			if let Some(ref mut task) = maybe_task {
				task.task_status = TaskStatusType::Stopped;
			}
		});
		TaskStatus::<Test>::insert(task_id, TaskStatusType::Stopped);

		// ❌ Bob is the task owner, but NOT the assigned miner
		assert_noop!(
			TaskManagementModule::confirm_miner_vacation(RuntimeOrigin::signed(bob), task_id),
			Error::<Test>::NotAssignedMiner
		);
	});
}

#[test]
fn fails_if_task_not_stopped() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
		assert_ok!(register_worker(alice, WorkerType::Executable, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			alice,
			0,
			Some(5),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Do not update status to Stopped → still Running
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id
		));

		// ❌ Cannot confirm vacation unless status is Stopped
		assert_noop!(
			TaskManagementModule::confirm_miner_vacation(RuntimeOrigin::signed(alice), task_id),
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
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		// Provide compute hours and register worker
		pallet_payment::ComputeHours::<Test>::insert(alice, 40);
		assert_ok!(register_worker(alice, WorkerType::Executable, "alice"));

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			alice,
			0,
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
		let task_kind_infer = TaskSubmissionData::OpenInference(OpenInferenceTask::Onnx(OnnxTask {
			storage_location_identifier: BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap(),
			triton_config: None
		}));

		pallet_payment::ComputeHours::<Test>::insert(alice, 30);
		assert_ok!(register_worker(alice, WorkerType::Executable, "alice"));

		// Schedule task and don't confirm reception (still Assigned)
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_kind_infer,
			alice,
			0,
			Some(15),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// ❌ Call stop while task is not Running
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

#[test]
fn test_register_and_retrieve_model_hash() {
	new_test_ext().execute_with(|| {
		use base64::engine::general_purpose::STANDARD;
		use base64::Engine;
		use hex_literal::hex;
		use sp_core::H256;

		let gatekeeper = 1u64;
		GatekeeperAccount::<Test>::put(gatekeeper);

		let origin = RuntimeOrigin::signed(gatekeeper);

		let model_id_vec =
			hex!("79c3bc0974696a2ea9efd2f7bca19fdd630834bd0086f1b4a1c3db3dce3b2a51").to_vec();

		let hash_b64 = "ecO8CXRpai6p79L3vKGf3WMINL0AhvG0ocPbPc47KlE=";
		let hash_bytes = STANDARD.decode(hash_b64).expect("Valid base64");
		assert_eq!(hash_bytes.len(), 32, "Hash must be 32 bytes");

		let model_hash = H256::from_slice(&hash_bytes);

		assert_ok!(TaskManagementModule::register_model_hash(
			origin.clone(),
			model_id_vec.clone(),
			model_hash
		));

		let mut model_id_fixed = [0u8; 32];
		model_id_fixed.copy_from_slice(&model_id_vec);

		let stored_hash = ModelHashes::<Test>::get(model_id_fixed);
		assert_eq!(stored_hash, Some(model_hash));
	});
}
