use crate::TaskStatusType;
use crate::{mock::*, Error};
pub use cyborg_primitives::worker::*;
use frame_support::BoundedVec;
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_std::convert::TryFrom;

#[test]
fn it_works_for_task_scheduler() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let executor = 2;
		let worker_latitude: Latitude = 590000;
		let worker_longitude:Longitude = 120000;
		let worker_ram: RamBytes = 100000000;
		let worker_storage: StorageBytes = 100000000;
		let worker_cpu: CpuCores = 12;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Register a worker for executor
		let api_info = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};

		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(executor),
			api_info.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Dispatch a signed extrinsic.
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_data.clone()
		));

		// Check task allocation and owner
		let task_id = TaskManagementModule::next_task_id() - 1;
		let assigned_worker = TaskManagementModule::task_allocations(task_id).unwrap();
		let task_owner = TaskManagementModule::task_owners(task_id).unwrap();
		assert_eq!(task_owner, alice);
		assert_eq!(executor, assigned_worker.0);

		// Check if task information is correct
		let task_info = TaskManagementModule::get_tasks(task_id).unwrap();
		assert_eq!(task_info.metadata, task_data);
		assert_eq!(task_info.task_owner, alice);
	});
}

#[test]
fn it_fails_when_no_workers_are_available() {
	new_test_ext().execute_with(|| {
		let alice = 1;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Dispatch a signed extrinsic and expect an error because no workers are available
		assert_noop!(
			TaskManagementModule::task_scheduler(RuntimeOrigin::signed(alice), task_data.clone()),
			Error::<Test>::NoWorkersAvailable
		);
	});
}

#[test]
fn it_works_for_submit_completed_task() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let worker_latitude: Latitude = 590000;
		let worker_longitude: Longitude = 120000;
		let worker_ram: RamBytes = 100000000;
		let worker_storage: StorageBytes = 100000000;
		let worker_cpu: CpuCores = 12;
		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Register a worker for Alice
		let api_info = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};

		// Register a worker for Bob
		let api_info_bob = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap(),
		};

		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			api_info.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(bob),
			api_info_bob.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Dispatch a signed extrinsic to schedule a task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_data.clone()
		));

		// Get the task_id of the scheduled task
		let task_id = TaskManagementModule::next_task_id() - 1;

		// Create a completed hash
		let completed_hash = H256::random();

		let result =
			BoundedVec::try_from(b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		// Dispatch a signed extrinsic to submit the completed task
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(alice),
			task_id,
			completed_hash,
			result
		));

		// Check task status
		let task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::PendingValidation);

		// Check task verifications
		let verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, alice);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
	});
}

#[test]
fn result_on_taskinfo_works_on_result_submit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(8);
		let alice = 1;
		let bob = 2;
		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some ipfs hash to executable".to_vec()).unwrap();

		// Register a worker for Alice
		let api_info = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};

		// Register a worker for Bob
		let api_info_bob = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap(),
		};

		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			api_info.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(bob),
			api_info_bob.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Dispatch a signed extrinsic to schedule a task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(bob),
			task_data.clone()
		));

		System::set_block_number(10);
		// Get the task_id of the scheduled task
		let task_id = TaskManagementModule::next_task_id() - 1;

		// Create a completed hash
		let completed_hash = H256::random();

		let result =
			BoundedVec::try_from(b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		// Dispatch a signed extrinsic to submit the completed task
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(bob),
			task_id,
			completed_hash,
			result.clone()
		));

		System::set_block_number(15);

		assert_eq!(
			TaskManagementModule::get_tasks(task_id)
				.unwrap()
				.result
				.unwrap(),
			result
		);

		// Check task status
		let task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::PendingValidation);

		// Check task verifications
		let verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, bob);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
	})
}

#[test]
fn it_fails_when_submit_completed_task_with_invalid_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let alice = 1;
		let bob = 2;
		let worker_latitude: Latitude = 590000;
		let worker_longitude: Longitude = 120000;
		let worker_ram: RamBytes = 100000000;
		let worker_storage: StorageBytes = 100000000;
		let worker_cpu: CpuCores = 12;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Register a worker for Alice
		let api_info = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};

		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			api_info.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Dispatch a signed extrinsic to schedule a task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(alice),
			task_data.clone()
		));

		// Get the task_id of the scheduled task
		let task_id = TaskManagementModule::next_task_id() - 1;

		// Create a completed hash
		let completed_hash = H256::random();

		let result =
			BoundedVec::try_from(b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		// Dispatch a signed extrinsic to submit the completed task with Bob as the sender
		assert_noop!(
			TaskManagementModule::submit_completed_task(
				RuntimeOrigin::signed(bob),
				task_id,
				completed_hash,
				result
			),
			Error::<Test>::InvalidTaskOwner
		);
	});
}

#[test]
fn it_works_when_verifying_task() {
	new_test_ext().execute_with(|| {
		let task_creator = 0;
		let executor = 1;
		let verifier = 2;
		let worker_latitude: Latitude = 590000;
		let worker_longitude: Longitude = 120000;
		let worker_ram: RamBytes = 100000000;
		let worker_storage: StorageBytes = 100000000;
		let worker_cpu: CpuCores = 12;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Register a worker for executor
		let api_info_executor = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(executor),
			api_info_executor.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Dispatch a signed extrinsic to schedule a task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(task_creator),
			task_data.clone()
		));

		// Get the task_id of the scheduled task
		let task_id = TaskManagementModule::next_task_id() - 1;

		// Create a completed hash
		let completed_hash = H256::random();

		// Register a worker for the verifier
		let api_info_verifier = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(verifier),
			api_info_verifier.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		let result =
			BoundedVec::try_from(b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		// Dispatch a signed extrinsic to submit the completed task by executor
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(executor),
			task_id,
			completed_hash,
			result
		));

		// Check task verifications
		let verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, executor);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));

		// Check task status
		let task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::PendingValidation);

		assert_ok!(TaskManagementModule::verify_completed_task(
			RuntimeOrigin::signed(verifier),
			task_id,
			completed_hash
		));

		let new_task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(new_task_status, TaskStatusType::Completed);
	});
}

#[test]
fn it_assigns_resolver_when_dispute_in_verification_and_resolves_task() {
	new_test_ext().execute_with(|| {
		let task_creator = 0;
		let executor = 1;
		let verifier = 2;
		let resolver = 3;
		let worker_latitude: Latitude = 590000;
		let worker_longitude: Longitude = 120000;
		let worker_ram: RamBytes = 100000000;
		let worker_storage: StorageBytes = 100000000;
		let worker_cpu: CpuCores = 12;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Register a worker for executor
		let api_info_executor = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(executor),
			api_info_executor.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Dispatch a signed extrinsic to schedule a task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(task_creator),
			task_data.clone()
		));

		// Get the task_id of the scheduled task
		let task_id = TaskManagementModule::next_task_id() - 1;

		// Create a completed hash
		let completed_hash = H256([123; 32]);

		// Register a worker for the verifier
		let api_info_verifier = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(verifier),
			api_info_verifier.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		let result =
			BoundedVec::try_from(b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		// Dispatch a signed extrinsic to submit the completed task by executor
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(executor),
			task_id,
			completed_hash,
			result
		));

		// Check task verifications
		let verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, executor);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));

		// Check task status
		let task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::PendingValidation);

		// Register a worker for the resolver
		let api_info_resolver = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(resolver),
			api_info_resolver.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Submit differing completed hash
		let completed_hash_2 = H256([222; 32]);

		assert_ok!(TaskManagementModule::verify_completed_task(
			RuntimeOrigin::signed(verifier),
			task_id,
			completed_hash_2
		));

		// Ensure task remains incompleted when invalid verification
		let mut new_task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(new_task_status, TaskStatusType::PendingValidation);

		// Check that task verification is now assigned a resolver
		let updated_verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(
			updated_verifications.resolver.clone().unwrap().account,
			resolver
		);

		assert_ok!(TaskManagementModule::resolve_completed_task(
			RuntimeOrigin::signed(resolver),
			task_id,
			completed_hash_2
		));

		// Check updated task status
		new_task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(new_task_status, TaskStatusType::Completed);
	});
}

#[test]
fn it_reassigns_task_when_resolver_fails_to_resolve() {
	new_test_ext().execute_with(|| {
		let task_creator = 0;
		let executor = 1;
		let verifier = 2;
		let resolver = 3;
		let worker_latitude: Latitude = 590000;
		let worker_longitude: Longitude = 120000;
		let worker_ram: RamBytes = 100000000;
		let worker_storage: StorageBytes = 100000000;
		let worker_cpu: CpuCores = 12;

		// Create a task data BoundedVec
		let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

		// Register a worker for executor
		let api_info_executor = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(executor),
			api_info_executor.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Dispatch a signed extrinsic to schedule a task
		assert_ok!(TaskManagementModule::task_scheduler(
			RuntimeOrigin::signed(task_creator),
			task_data.clone()
		));

		// Get the task_id of the scheduled task
		let task_id = TaskManagementModule::next_task_id() - 1;

		// Create a completed hash
		let completed_hash = H256([123; 32]);

		// Register a worker for the verifier
		let api_info_verifier = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(verifier),
			api_info_verifier.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		let result =
			BoundedVec::try_from(b"Qmaf1xjXDY7fhY9QQw5XfwdkYZQ2cPhaZRT2TfXeadYCbD".to_vec()).unwrap();

		// Dispatch a signed extrinsic to submit the completed task by executor
		assert_ok!(TaskManagementModule::submit_completed_task(
			RuntimeOrigin::signed(executor),
			task_id,
			completed_hash,
			result
		));

		// Check task verifications
		let verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, executor);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));

		// Check task status
		let task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::PendingValidation);

		// Register a worker for the resolver
		let api_info_resolver = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker3.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(resolver),
			api_info_resolver.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Submit differing completed hash
		let completed_hash_2 = H256([222; 32]);

		assert_ok!(TaskManagementModule::verify_completed_task(
			RuntimeOrigin::signed(verifier),
			task_id,
			completed_hash_2
		));

		// Ensure task remains incompleted when invalid verification
		let mut new_task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(new_task_status, TaskStatusType::PendingValidation);

		// Check that task verification is now assigned a resolver
		let updated_verifications = TaskManagementModule::task_verifications(task_id).unwrap();
		assert_eq!(updated_verifications.resolver.unwrap().account, resolver);

		// Submit differing completed hash
		let completed_hash_3 = H256([111; 32]);

		// fails if no new workers differing from workers participating in this task
		assert_noop!(
			TaskManagementModule::resolve_completed_task(
				RuntimeOrigin::signed(resolver),
				task_id,
				completed_hash_3
			),
			Error::<Test>::NoNewWorkersAvailable
		); // TODO: Redesign as this is potential bug for retrys that match false completed hashes

		let new_executor = 4;

		// Register a worker for the new executor
		let api_info_new_executor = WorkerAPI {
			domain: BoundedVec::try_from(b"https://api-worker4.testing".to_vec()).unwrap(),
		};
		assert_ok!(edgeConnectModule::register_worker(
			RuntimeOrigin::signed(new_executor),
			api_info_new_executor.domain,
			worker_latitude,
			worker_longitude,
			worker_ram,
			worker_storage,
			worker_cpu
		));

		// Reassigns a new executor when resolver cannot find a matching completed hash
		assert_ok!(TaskManagementModule::resolve_completed_task(
			RuntimeOrigin::signed(resolver),
			task_id,
			completed_hash_3
		));

		// Check updated task status
		new_task_status = TaskManagementModule::task_status(task_id).unwrap();
		assert_eq!(new_task_status, TaskStatusType::Assigned);

		// Check task allocations for new executor for task
		let task_allocated_to = TaskManagementModule::task_allocations(task_id).unwrap();
		assert_eq!(task_allocated_to.0, new_executor);

		// Ensure task verifications are empty
		let updated_verifications_after_reassignment =
			TaskManagementModule::task_verifications(task_id);
		assert_eq!(updated_verifications_after_reassignment, None);
	});
}
