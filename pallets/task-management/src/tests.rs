use crate::{mock::*, Error};
use crate::{NextTaskId, TaskAllocations, TaskOwners, TaskStatus, Tasks,ComputeAggregations};
use frame_support::traits::Task;
use frame_support::{assert_noop, assert_ok};

pub use cyborg_primitives::task::{TaskStatusType, TaskType,TaskKind,TaskInfo};
pub use cyborg_primitives::worker::*;
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use frame_support::BoundedVec;
use sp_core::H256;
use frame_system::Origin;
use sp_std::convert::TryFrom;
use frame_system::pallet_prelude::BlockNumberFor;

fn register_worker(
	account: u64,
	worker_type: WorkerType,
	domain_str: &str,
) -> Result<PostDispatchInfo, DispatchErrorWithPostInfo> {
	EdgeConnectModule::register_worker(
		RuntimeOrigin::signed(account),
		worker_type,
		BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap(),
		590000,
		120000,
		10000000,
		10000000,
		12,
	)
}

#[test]
#[test]
fn it_works_for_task_scheduler() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;

		let task_type_docker = TaskType::Docker;
		let task_type_exec = TaskType::Executable;
		let task_kind_neurozk = TaskKind::NeuroZK;
		let task_kind_infer = TaskKind::OpenInference;

		let worker_id_docker = 0;
		let worker_id_exec = 1;

		// Provide initial compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 30);

		// Task metadata (e.g., docker image or model)
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// zk_files_cid only required for NeuroZK
		let zk_files_cid = Some(
			BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap()
		);

		// Register worker for Docker and Executable
		assert_ok!(register_worker(executor, WorkerType::Docker, "executor"));
		assert_ok!(register_worker(executor, WorkerType::Executable, "executor"));

		// --------------------------------------------------
		// ✅ Schedule OpenInference Docker Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type_docker.clone(),
			task_kind_infer.clone(),
			task_data.clone(),
			None,
			executor,
			worker_id_docker,
			Some(10)
		));

		let task_id_0 = NextTaskId::<Test>::get() - 1;
		let task_info_0 = Tasks::<Test>::get(task_id_0).unwrap();
		assert_eq!(task_info_0.task_kind, TaskKind::OpenInference);
		assert_eq!(task_info_0.task_type, TaskType::Docker);
		assert_eq!(task_info_0.zk_files_cid, None);

		// --------------------------------------------------
		// ✅ Schedule OpenInference Executable Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type_exec.clone(),
			task_kind_infer.clone(),
			task_data.clone(),
			None,
			executor,
			worker_id_exec,
			Some(10)
		));

		let task_id_1 = NextTaskId::<Test>::get() - 1;
		let task_info_1 = Tasks::<Test>::get(task_id_1).unwrap();
		assert_eq!(task_info_1.task_kind, TaskKind::OpenInference);
		assert_eq!(task_info_1.task_type, TaskType::Executable);
		assert_eq!(task_info_1.zk_files_cid, None);

		// --------------------------------------------------
		// ✅ Schedule NeuroZK Executable Task (valid with zk_files)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type_exec.clone(),
			task_kind_neurozk,
			task_data.clone(),
			zk_files_cid.clone(),
			executor,
			worker_id_exec,
			Some(10)
		));

		let task_id_2 = NextTaskId::<Test>::get() - 1;
		let task_info_2 = Tasks::<Test>::get(task_id_2).unwrap();
		assert_eq!(task_info_2.task_kind, TaskKind::NeuroZK);
		assert_eq!(task_info_2.task_type, TaskType::Executable);
		assert_eq!(task_info_2.zk_files_cid, zk_files_cid);
	});
}

#[test]
fn it_fails_when_worker_not_registered() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let task_type_docker = TaskType::Docker;
		let task_kind_neurozk = TaskKind::NeuroZK;
		let worker_owner = 2;
		let worker_id = 99;
		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
		// Register a worker for executor
		let api_info = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};
		// Create zk_files_cid
		let zk_files_cid =
			BoundedVec::try_from(b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap();

		// Provide an initial compute hours balance for Alice
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		assert_ok!(register_worker(alice, WorkerType::Executable, "executor"));

		// Dispatch a signed extrinsic.
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_type_docker,
				task_kind_neurozk,
				task_data.clone(),
				Some(zk_files_cid.clone()),
				worker_owner,
				worker_id,
				Some(1),
			),
			Error::<Test>::WorkerDoesNotExist
		);
	});
}

#[test]
fn it_fails_when_no_workers_are_available() {
	new_test_ext().execute_with(|| {
		let alice = 1;
		let worker_owner = 2;
		let worker_id = 0;
		let task_type_docker = TaskType::Docker;
		let task_kind_infer = TaskKind::OpenInference;
		// Provide an initial compute hours balance for Alice
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_type_docker,
				task_kind_infer,
				task_data.clone(),
				None,
				worker_owner,
				worker_id,
				Some(10)
			),
			Error::<Test>::NoWorkersAvailable
		);
	});
}

#[test]
fn it_fails_when_no_computer_hours_available() {
	new_test_ext().execute_with(|| {
		let alice = 1;

		let worker_owner = 2;
		let worker_id = 0;
		let task_type_docker = TaskType::Docker;
		let task_kind_infer = TaskKind::OpenInference;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
				task_type_docker,
				task_kind_infer,
				task_data.clone(),
				None,
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
		System::set_block_number(1);
		let creator = 1;
		let executor = 2;
		let worker_id = 0;
		let task_type = TaskType::Executable;
		let task_kind = TaskKind::OpenInference;
		let task_data = BoundedVec::truncate_from(b"model.bin".to_vec());

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			task_kind,
			task_data.clone(),
			None,
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
		let creator = 1;
		let executor = 2;
		let intruder = 99;
		let worker_id = 0;
		let task_type = TaskType::Executable;

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			TaskKind::OpenInference,
			task_data.clone(),
			None,
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
		let creator = 1;
		let executor = 2;
		let worker_id = 0;
		let task_type = TaskType::Executable;

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			TaskKind::OpenInference,
			task_data.clone(),
			None,
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
fn submit_completed_task_should_work_for_valid_executor() {
	new_test_ext().execute_with(|| {
		let creator = 1;
		let executor = 2;
		let verifier = 3;
		let worker_id = 0;
		let task_type = TaskType::Executable;

		System::set_block_number(1);

		// Setup: compute hours, workers, task
		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));
		assert_ok!(register_worker(verifier, WorkerType::Executable, "verifier"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			TaskKind::OpenInference,
			task_data,
			None,
			executor,
			worker_id,
			Some(10)
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Confirm reception
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(executor),
			task_id
		));

		// Now submit completed task
		let completed_hash = H256::random();
		let result = BoundedVec::truncate_from(b"output.bin".to_vec());
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(executor),
			task_id,
			completed_hash,
			result.clone()
		));

		// Check task status and result
		let task = Tasks::<Test>::get(task_id).unwrap();
		assert_eq!(task.task_status, TaskStatusType::PendingValidation);
		assert_eq!(task.result, Some(result.clone()));

		// ✅ Check verification record using the public getter
		let ver = TaskManagementModule::get_task_verifications(task_id).unwrap();
		assert_eq!(ver.executor.account, executor);
		assert_eq!(ver.executor.completed_hash, Some(completed_hash));
		assert!(ver.verifier.is_some());
	});
}
#[test]
fn submit_completed_task_should_fail_for_unassigned_worker() {
	new_test_ext().execute_with(|| {
		let creator = 1;
		let executor = 2;
		let intruder = 99;
		let worker_id = 0;
		let task_type = TaskType::Executable;

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));
		assert_ok!(register_worker(intruder, WorkerType::Executable, "bad"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			TaskKind::OpenInference,
			task_data,
			None,
			executor,
			worker_id,
			Some(10)
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Intruder tries to submit completed task
		let result = BoundedVec::truncate_from(b"output".to_vec());
		let hash = H256::random();
		assert_noop!(
			TaskManagementModule::submit_completed_task(
				RuntimeOrigin::signed(intruder),
				task_id,
				hash,
				result
			),
			Error::<Test>::InvalidTaskOwner
		);
	});
}

#[test]
fn submit_completed_task_should_fail_if_task_not_running() {
	new_test_ext().execute_with(|| {
		let creator = 1;
		let executor = 2;
		let worker_id = 0;
		let task_type = TaskType::Executable;

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Executable, "exec"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			TaskKind::OpenInference,
			task_data,
			None,
			executor,
			worker_id,
			Some(10)
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// Try to submit result before confirm_task_reception
		let result = BoundedVec::truncate_from(b"output".to_vec());
		let hash = H256::random();
		assert_noop!(
			TaskManagementModule::submit_completed_task(
				RuntimeOrigin::signed(executor),
				task_id,
				hash,
				result
			),
			Error::<Test>::RequireAssignedTask
		);
	});
}



#[test]
fn it_works_for_submit_completed_task() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;

		let task_type_docker = TaskType::Docker;
		let task_type_exec = TaskType::Executable;

		let task_kind_zk = TaskKind::NeuroZK;
		let task_kind_infer = TaskKind::OpenInference;

		let worker_id_0 = 0;
		let worker_id_1 = 1;

		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
		let zk_files_cid = Some(BoundedVec::try_from(
			b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()
		).unwrap());

		let result = BoundedVec::try_from(
			b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()
		).unwrap();

		let completed_hash = H256::random();

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 40);

		// Register Docker workers
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));
		assert_ok!(register_worker(bob, WorkerType::Docker, "bob"));

		// 🔹 Submit Docker + OpenInference Task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type_docker,
			task_kind_infer,
			task_data.clone(),
			None, // OpenInference → no zk_files_cid
			alice,
			worker_id_0,
			Some(10),
		));

		let task_id_0 = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id_0,
		));

		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(alice),
			task_id_0,
			completed_hash,
			result.clone()
		));

		let status_0 = TaskStatus::<Test>::get(task_id_0).unwrap();
		assert_eq!(status_0, TaskStatusType::PendingValidation);

		let ver_0 = TaskManagementModule::get_task_verifications(task_id_0).unwrap();
		assert_eq!(ver_0.executor.account, alice);
		assert_eq!(ver_0.executor.completed_hash, Some(completed_hash));

		// 🔹 Submit Executable + NeuroZK Task (requires zk_files_cid!)
		assert_ok!(register_worker(alice, WorkerType::Executable, "alice"));
		assert_ok!(register_worker(bob, WorkerType::Executable, "bob"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type_exec,
			task_kind_zk,
			task_data.clone(),
			zk_files_cid.clone(), // NeuroZK → must be Some
			alice,
			worker_id_1,
			Some(10),
		));

		let task_id_1 = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(alice),
			task_id_1,
		));

		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(alice),
			task_id_1,
			completed_hash,
			result.clone()
		));

		let status_1 = TaskStatus::<Test>::get(task_id_1).unwrap();
		assert_eq!(status_1, TaskStatusType::PendingValidation);

		let ver_1 = TaskManagementModule::get_task_verifications(task_id_1).unwrap();
		assert_eq!(ver_1.executor.account, alice);
		assert_eq!(ver_1.executor.completed_hash, Some(completed_hash));
	});
}


#[test]
fn result_on_taskinfo_works_on_result_submit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(8);
		let alice = 1;
		let bob = 2;

		let worker_type_docker = WorkerType::Docker;
		let worker_type_exec = WorkerType::Executable;
		let task_type_docker = TaskType::Docker;
		let task_type_exec = TaskType::Executable;

		let task_kind_zk = TaskKind::NeuroZK;
		let task_kind_infer = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"some ipfs hash to executable".to_vec()).unwrap();
		let zk_files_cid = Some(BoundedVec::try_from(
			b"Qmf9v8VbJ6WFGbakeWEXFhUc91V1JG26grakv3dTj8rERh".to_vec()).unwrap());

		let result = BoundedVec::try_from(
			b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		let completed_hash = H256::random();

		let worker_id_0 = 0;
		let worker_id_1 = 1;

		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100_000_000;
		let storage: StorageBytes = 100_000_000;
		let cpu: CpuCores = 12;

		let api_info = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};
		let api_info_bob = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap(),
		};

		// Register Docker workers
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_docker.clone(),
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(bob),
			worker_type_docker.clone(),
			api_info_bob.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Provide balance
		pallet_payment::ComputeHours::<Test>::insert(bob, 20);

		// 🧪 Schedule Docker + OpenInference task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(bob),
			task_type_docker,
			task_kind_infer,
			task_data.clone(),
			None,
			bob,
			worker_id_0,
			Some(10)
		));

		let task_id_0 = NextTaskId::<Test>::get() - 1;

		// 🧪 Confirm reception before submitting
		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			task_id_0
		));

		// 🧪 Submit result
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(bob),
			task_id_0,
			completed_hash,
			result.clone()
		));

		System::set_block_number(15);
		assert_eq!(Tasks::<Test>::get(task_id_0).unwrap().result.unwrap(), result);
		assert_eq!(TaskStatus::<Test>::get(task_id_0).unwrap(), TaskStatusType::PendingValidation);

		let verifications = TaskManagementModule::get_task_verifications(task_id_0).unwrap();
		assert_eq!(verifications.executor.account, bob);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));

		// Register Executable workers for ZK
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_exec.clone(),
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(bob),
			worker_type_exec.clone(),
			api_info_bob.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// 🧪 Schedule Executable + NeuroZK task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(bob),
			task_type_exec,
			task_kind_zk,
			task_data.clone(),
			zk_files_cid.clone(), // required for ZK
			bob,
			worker_id_1,
			Some(10)
		));

		let task_id_1 = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(
			RuntimeOrigin::signed(bob),
			task_id_1
		));

		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(bob),
			task_id_1,
			completed_hash,
			result.clone()
		));

		assert_eq!(Tasks::<Test>::get(task_id_1).unwrap().result.unwrap(), result);
		assert_eq!(TaskStatus::<Test>::get(task_id_1).unwrap(), TaskStatusType::PendingValidation);

		let verifications = TaskManagementModule::get_task_verifications(task_id_1).unwrap();
		assert_eq!(verifications.executor.account, bob);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
	});
}

#[test]
fn verify_completed_task_should_succeed_with_correct_hash() {
	new_test_ext().execute_with(|| {
		let creator = 1;
		let executor = 2;
		let verifier = 3;
		let worker_id = 0;
		let task_type = TaskType::Docker;

		pallet_payment::ComputeHours::<Test>::insert(creator, 50);
		assert_ok!(register_worker(executor, WorkerType::Docker, "executor"));
		assert_ok!(register_worker(verifier, WorkerType::Docker, "verifier"));

		let task_data = BoundedVec::truncate_from(b"model_v1".to_vec());

		// Schedule
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			task_type,
			TaskKind::OpenInference,
			task_data.clone(),
			None,
			executor,
			worker_id,
			Some(10)
		));
		let task_id = NextTaskId::<Test>::get() - 1;

		// Confirm reception + submit result
		assert_ok!(TaskManagementModule::confirm_task_reception(RuntimeOrigin::signed(executor), task_id));
		let result = BoundedVec::truncate_from(b"result_v1".to_vec());
		let hash = H256::random();
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(executor),
			task_id,
			hash,
			result.clone()
		));

		// Extract the assigned verifier
		let verifications = TaskManagementModule::get_task_verifications(task_id).unwrap();
		let assigned_verifier = verifications.verifier.as_ref().unwrap().account.clone();

		// Verifier confirms correct hash
		assert_ok!(TaskManagementModule::verify_completed_task(
			RuntimeOrigin::signed(assigned_verifier.clone()),
			task_id,
			hash
		));

		// Task should be marked completed
		assert_eq!(TaskStatus::<Test>::get(task_id).unwrap(), TaskStatusType::Completed);
	});
}


#[test]
fn verify_completed_task_should_trigger_resolver_on_hash_mismatch() {
	new_test_ext().execute_with(|| {
		let creator = 1;
		let executor = 2;
		let verifier = 3;
		let resolver_candidate = 4;
		let worker_id = 0;

		pallet_payment::ComputeHours::<Test>::insert(creator, 50);
		assert_ok!(register_worker(executor, WorkerType::Docker, "executor"));
		assert_ok!(register_worker(verifier, WorkerType::Docker, "verifier"));
		assert_ok!(register_worker(resolver_candidate, WorkerType::Docker, "resolver"));

		let task_data = BoundedVec::truncate_from(b"file.zkrun".to_vec());

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
			TaskType::Docker,
			TaskKind::OpenInference,
			task_data.clone(),
			None,
			executor,
			worker_id,
			Some(10)
		));
		let task_id = NextTaskId::<Test>::get() - 1;

		assert_ok!(TaskManagementModule::confirm_task_reception(RuntimeOrigin::signed(executor), task_id));
		let correct_hash = H256::random();
		let wrong_hash = H256::random();
		let result = BoundedVec::truncate_from(b"prediction-123".to_vec());

		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(executor),
			task_id,
			correct_hash,
			result
		));

		let verifications = TaskManagementModule::get_task_verifications(task_id).unwrap();
		let assigned_verifier = verifications.verifier.unwrap().account;

		// Verifier submits incorrect hash → should assign resolver
		assert_ok!(TaskManagementModule::verify_completed_task(
			RuntimeOrigin::signed(assigned_verifier.clone()),
			task_id,
			wrong_hash
		));

		let updated_verifications = TaskManagementModule::get_task_verifications(task_id).unwrap();
		assert!(updated_verifications.resolver.is_some());
		assert_eq!(TaskStatus::<Test>::get(task_id).unwrap(), TaskStatusType::PendingValidation);
	});
}

#[test]
fn it_works_for_confirm_miner_vacation() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let task_type = TaskType::Docker;
		let task_kind = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"docker-alice-vacation".to_vec()).unwrap();

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Register a Docker worker
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		// 🔹 Submit task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type.clone(),
			task_kind.clone(),
			task_data.clone(),
			None,
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
fn fails_if_not_task_owner_for_vacation() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let task_type = TaskType::Docker;
		let task_kind = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"docker-alice-task".to_vec()).unwrap();

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type.clone(),
			task_kind.clone(),
			task_data.clone(),
			None,
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

		// ❌ Bob is not the task owner
		assert_noop!(
			TaskManagementModule::confirm_miner_vacation(RuntimeOrigin::signed(bob), task_id),
			Error::<Test>::NotTaskOwner
		);
	});
}

#[test]
fn fails_if_task_not_stopped() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let task_type = TaskType::Docker;
		let task_kind = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"docker-alice-not-stopped".to_vec()).unwrap();

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type.clone(),
			task_kind.clone(),
			task_data.clone(),
			None,
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
        System::set_block_number(1);
        let alice = 1;

        let task_type = TaskType::Docker;
        let task_kind = TaskKind::OpenInference;
        let metadata = BoundedVec::try_from(b"docker-image-task-v1.0".to_vec()).unwrap();

        // Provide compute hours and register worker
        pallet_payment::ComputeHours::<Test>::insert(alice, 40);
        assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

        // Schedule task
        assert_ok!(TaskManagementModule::task_scheduler(
            RuntimeOrigin::signed(alice),
            task_type,
            task_kind,
            metadata.clone(),
            None,
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
		System::set_block_number(1);
		let alice = 1;
		let task_type = TaskType::Docker;
		let task_kind = TaskKind::OpenInference;
		let task_data = BoundedVec::try_from(b"some-task-not-running".to_vec()).unwrap();

		pallet_payment::ComputeHours::<Test>::insert(alice, 30);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		// Schedule task and don't confirm reception (still Assigned)
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_type.clone(),
			task_kind.clone(),
			task_data.clone(),
			None,
			alice,
			0,
			Some(15),
		));

		let task_id = NextTaskId::<Test>::get() - 1;

		// ❌ Call stop while task is not Running
		assert_noop!(
			TaskManagementModule::stop_task_and_vacate_miner(
				RuntimeOrigin::signed(alice),
				task_id
			),
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
			TaskManagementModule::stop_task_and_vacate_miner(
				RuntimeOrigin::signed(1),
				task_id
			),
			Error::<Test>::TaskNotFound
		);
	});
}


