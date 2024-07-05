use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_support::BoundedVec;
use sp_core::H256;
use sp_std::convert::TryFrom;

#[test]
fn it_works_for_task_scheduler() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let alice = 1;
        
        // Create a task data BoundedVec
        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

        // Register a worker for Alice
        let api_info = pallet_worker_clusters::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };
        
        assert_ok!(WorkerClusters::register_worker(
            RuntimeOrigin::signed(alice),
            api_info.ip,
            api_info.domain
        ));

        // Dispatch a signed extrinsic.
        assert_ok!(TaskManagement::task_scheduler(RuntimeOrigin::signed(alice), task_data.clone()));

        // Check task allocation and owner
        let task_id = TaskManagement::next_task_id() - 1;
        let assigned_worker = TaskManagement::task_allocations(task_id).unwrap();
        let task_owner = TaskManagement::task_owners(task_id).unwrap();
        assert_eq!(task_owner, alice);

        // Check if task information is correct
        let task_info = TaskManagement::get_tasks(task_id).unwrap();
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
            TaskManagement::task_scheduler(RuntimeOrigin::signed(alice), task_data.clone()),
            Error::<Test>::NoWorkersAvailable
        );
    });
}

#[test]
fn it_works_for_submit_completed_task() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let alice = 1;

        // Create a task data BoundedVec
        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

        // Register a worker for Alice
        let api_info = pallet_worker_clusters::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };

        assert_ok!(WorkerClusters::register_worker(
            RuntimeOrigin::signed(alice),
            api_info.ip,
            api_info.domain
        ));

        // Dispatch a signed extrinsic to schedule a task
        assert_ok!(TaskManagement::task_scheduler(RuntimeOrigin::signed(alice), task_data.clone()));

        // Get the task_id of the scheduled task
        let task_id = TaskManagement::next_task_id() - 1;

        // Create a completed hash
        let completed_hash = H256::random();

        // Dispatch a signed extrinsic to submit the completed task
        assert_ok!(TaskManagement::submit_completed_task(RuntimeOrigin::signed(alice), task_id, completed_hash));

        // Check task status
        let task_status = TaskManagement::task_status(task_id).unwrap();
        assert_eq!(task_status, TaskStatusType::PendingValidation);

        // Check task verifications
        let verifications = TaskManagement::task_verifications(task_id).unwrap();
        assert_eq!(verifications.executor.account, alice);
        assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
    });
}

#[test]
fn it_fails_when_submit_completed_task_with_invalid_owner() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let alice = 1;
        let bob = 2;

        // Create a task data BoundedVec
        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();

        // Register a worker for Alice
        let api_info = pallet_worker_clusters::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };

        assert_ok!(WorkerClusters::register_worker(
            RuntimeOrigin::signed(alice),
            api_info.ip,
            api_info.domain
        ));

        // Dispatch a signed extrinsic to schedule a task
        assert_ok!(TaskManagement::task_scheduler(RuntimeOrigin::signed(alice), task_data.clone()));

        // Get the task_id of the scheduled task
        let task_id = TaskManagement::next_task_id() - 1;

        // Create a completed hash
        let completed_hash = H256::random();

        // Dispatch a signed extrinsic to submit the completed task with Bob as the sender
        assert_noop!(
            TaskManagement::submit_completed_task(RuntimeOrigin::signed(bob), task_id, completed_hash),
            Error::<Test>::InvalidTaskOwner
        );
    });
}