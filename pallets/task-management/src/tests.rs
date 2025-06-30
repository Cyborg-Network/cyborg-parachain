use crate::{mock::*, Error};
use crate::{ComputeAggregations, NextTaskId, TaskStatus, Tasks};
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
			WorkerType::Docker,
			"docker.worker"
		));
		assert_ok!(register_worker(
			executor,
			WorkerType::Executable,
			"exec.worker"
		));

		// Verify workers are registered
		assert!(pallet_edge_connect::WorkerClusters::<Test>::contains_key((
			executor, 0
		)));
		assert!(pallet_edge_connect::ExecutableWorkers::<Test>::contains_key((executor, 1)));

		let task_kind_neurozk = TaskKind::NeuroZK;
		let task_kind_infer = TaskKind::OpenInference;

		let worker_id_docker = 0;
		let worker_id_exec = 1;

		// Provide initial compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 30);

		// Task metadata (e.g., docker image or model)
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// zk_files_cid only required for NeuroZK
		let zk_files_cid = Some(NeuroZkTaskSubmissionDetails {
			zk_input: BoundedVec::try_from(b"input".to_vec()).unwrap(),
			zk_settings: BoundedVec::try_from(b"settings".to_vec()).unwrap(),
			zk_verifying_key: BoundedVec::try_from(b"verifying_key".to_vec()).unwrap(),
		});

		// --------------------------------------------------
		// ✅ Schedule OpenInference Docker Task (valid)
		// --------------------------------------------------
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
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
		assert_eq!(task_info_0.nzk_data, None);

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
			task_data.clone(),
			zk_files_cid.clone(),
			executor,
			worker_id_exec,
			Some(10)
		));

		let task_id_2 = NextTaskId::<Test>::get() - 1;
		let task_info_2 = Tasks::<Test>::get(task_id_2).unwrap();
		assert_eq!(task_info_2.task_kind, TaskKind::NeuroZK);
		assert!(task_info_2.nzk_data.is_some());
		if let Some(nzk_data) = task_info_2.nzk_data {
			assert_eq!(nzk_data.zk_input, zk_files_cid.as_ref().unwrap().zk_input);
			assert_eq!(
				nzk_data.zk_settings,
				zk_files_cid.as_ref().unwrap().zk_settings
			);
			assert_eq!(
				nzk_data.zk_verifying_key,
				zk_files_cid.as_ref().unwrap().zk_verifying_key
			);
		}
	});
}

#[test]
fn it_fails_when_worker_not_registered() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let task_kind_neurozk = TaskKind::NeuroZK;
		let worker_owner = 2;
		let worker_id = 99;

		// Register an Executable worker to ensure workers exist
		assert_ok!(register_worker(
			worker_owner,
			WorkerType::Executable,
			"exec.worker"
		));

		// Create task data and zk_files_cid
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
		let zk_files_cid = NeuroZkTaskSubmissionDetails {
			zk_input: BoundedVec::try_from(b"input".to_vec()).unwrap(),
			zk_settings: BoundedVec::try_from(b"settings".to_vec()).unwrap(),
			zk_verifying_key: BoundedVec::try_from(b"verifying_key".to_vec()).unwrap(),
		};

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Attempt to schedule with non-existent worker ID
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		let alice = 1;
		let worker_owner = 2;
		let worker_id = 0;
		let task_kind_infer = TaskKind::OpenInference;
		// Provide an initial compute hours balance for Alice
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Ensure no workers exist
		assert!(pallet_edge_connect::WorkerClusters::<Test>::iter()
			.next()
			.is_none());
		assert!(pallet_edge_connect::ExecutableWorkers::<Test>::iter()
			.next()
			.is_none());

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		let alice = 1;

		let worker_owner = 2;
		let worker_id = 0;
		let task_kind_infer = TaskKind::OpenInference;

		// Register worker first
		assert_ok!(register_worker(
			worker_owner,
			WorkerType::Docker,
			"worker.domain"
		));

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(
				RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		System::set_block_number(1);
		let creator = 1;
		let executor = 2;
		let worker_id = 0;
		let task_kind = TaskKind::OpenInference;
		let task_data = BoundedVec::truncate_from(b"model.bin".to_vec());

		// Register worker first
		assert_ok!(register_worker(executor, WorkerType::Docker, "exec"));

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
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
		setup_gatekeeper();
		let creator = 1;
		let executor = 2;
		let intruder = 99;
		let worker_id = 0;

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Docker, "exec"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
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
		setup_gatekeeper();
		let creator = 1;
		let executor = 2;
		let worker_id = 0;

		pallet_payment::ComputeHours::<Test>::insert(creator, 100);
		assert_ok!(register_worker(executor, WorkerType::Docker, "exec"));

		let task_data = BoundedVec::truncate_from(b"task".to_vec());
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(creator),
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
fn it_works_for_confirm_miner_vacation() {
	new_test_ext().execute_with(|| {
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let task_kind = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"docker-alice-vacation".to_vec()).unwrap();

		// Provide compute hours
		pallet_payment::ComputeHours::<Test>::insert(alice, 20);

		// Register a Docker worker
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		// 🔹 Submit task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let task_kind = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"docker-alice-task".to_vec()).unwrap();

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let task_kind = TaskKind::OpenInference;

		let task_data = BoundedVec::try_from(b"docker-alice-not-stopped".to_vec()).unwrap();

		pallet_payment::ComputeHours::<Test>::insert(alice, 10);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;

		let task_kind = TaskKind::OpenInference;
		let metadata = BoundedVec::try_from(b"docker-image-task-v1.0".to_vec()).unwrap();

		// Provide compute hours and register worker
		pallet_payment::ComputeHours::<Test>::insert(alice, 40);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		// Schedule task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
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
		setup_gatekeeper();
		System::set_block_number(1);
		let alice = 1;
		let task_kind = TaskKind::OpenInference;
		let task_data = BoundedVec::try_from(b"some-task-not-running".to_vec()).unwrap();

		pallet_payment::ComputeHours::<Test>::insert(alice, 30);
		assert_ok!(register_worker(alice, WorkerType::Docker, "alice"));

		// Schedule task and don't confirm reception (still Assigned)
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
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
