//! Benchmarking setup for pallet-task-management
use super::*;
use crate::{Pallet as TaskManagementModule, Config};
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use frame_support::BoundedVec;
use sp_core::H256;
use frame_system::RawOrigin;

benchmarks! {
    task_scheduler {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        
        // Register a worker for the caller
        let api_info = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(caller.clone()).into(), api_info.ip, api_info.domain)?;

        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
    }: _(RawOrigin::Signed(caller.clone()), task_data.clone())
    verify {
        let task_id = TaskManagementModule::<T>::next_task_id() - 1;
        let task_info = TaskManagementModule::<T>::get_tasks(task_id).unwrap();
        assert_eq!(task_info.metadata, task_data);
        assert_eq!(task_info.task_owner, caller);
    }

    submit_completed_task {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();

        // Register a worker for the executor
        let executor: T::AccountId = account("executor", 0, 0);
        let api_info = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(executor.clone()).into(), api_info.ip, api_info.domain)?;

        // Create a task data
        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
        TaskManagementModule::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;

        // Register a worker for the verifier
        let verifier: T::AccountId = account("verifier", 0, 0);
        let api_info_verifier = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(verifier.clone()).into(), api_info_verifier.ip, api_info_verifier.domain)?;

        let task_id = TaskManagementModule::<T>::next_task_id() - 1;
        let completed_hash = H256([123; 32]);
    }: _(RawOrigin::Signed(executor.clone()), task_id, completed_hash)
    verify {
        let task_status = TaskManagementModule::<T>::task_status(task_id).unwrap();
        assert_eq!(task_status, TaskStatusType::PendingValidation);

        let verifications = TaskManagementModule::<T>::task_verifications(task_id).unwrap();
        assert_eq!(verifications.executor.account, executor);
        assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
        assert_eq!(verifications.verifier.clone().unwrap().account, verifier);
        assert_eq!(verifications.verifier.clone().unwrap().completed_hash, None);
    }

    verify_completed_task {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();
        let executor: T::AccountId = account("executor", 0, 0);
        let verifier: T::AccountId = account("verifier", 0, 0);

        // Register a worker for the executor
        let api_info_executor = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(executor.clone()).into(), api_info_executor.ip, api_info_executor.domain)?;

        // Create a task data
        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
        TaskManagementModule::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;

        // Register a worker for the verifier
        let api_info_verifier = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(verifier.clone()).into(), api_info_verifier.ip, api_info_verifier.domain)?;

        let task_id = TaskManagementModule::<T>::next_task_id() - 1;

        let completed_hash = H256([123; 32]);
        TaskManagementModule::<T>::submit_completed_task(RawOrigin::Signed(executor.clone()).into(), task_id, completed_hash)?;

    }: _(RawOrigin::Signed(verifier.clone()), task_id, completed_hash)
    verify {
        let task_status = TaskManagementModule::<T>::task_status(task_id).unwrap();
        assert_eq!(task_status, TaskStatusType::Completed);
    }

    resolve_completed_task {
        let s in 0 .. 100;
        let caller: T::AccountId = whitelisted_caller();

        let executor: T::AccountId = account("executor", 0, 0);
        let verifier: T::AccountId = account("verifier", 0, 0);
        let resolver: T::AccountId = account("resolver", 0, 0);

        // Register a worker for the executor
        let api_info_executor = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(executor.clone()).into(), api_info_executor.ip, api_info_executor.domain)?;
        
        // Create a task data
        let task_data = BoundedVec::try_from(b"some-docker-imgv.0".to_vec()).unwrap();
        TaskManagementModule::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;
        let task_id = TaskManagementModule::<T>::next_task_id() - 1;

        // Register a worker for the verifier
        let api_info_verifier = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker2.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(verifier.clone()).into(), api_info_verifier.ip, api_info_verifier.domain)?;
        
        // Submit completed task by the executor
        let completed_hash = H256([4; 32]);
        TaskManagementModule::<T>::submit_completed_task(RawOrigin::Signed(executor.clone()).into(), task_id, completed_hash)?;

        // Register a worker for the resolver
        let api_info_resolver = pallet_edge_connect::types::WorkerAPI {
            ip: None,
            domain: Some(BoundedVec::try_from(b"https://api-worker3.testing".to_vec()).unwrap())
        };
        pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(resolver.clone()).into(), api_info_resolver.ip, api_info_resolver.domain)?;

        let completed_differing_hash = H256([123; 32]);
        // Verify completed task by the verifier
        TaskManagementModule::<T>::verify_completed_task(RawOrigin::Signed(verifier.clone()).into(), task_id, completed_differing_hash)?;

        let verifications = TaskManagementModule::<T>::task_verifications(task_id).unwrap();
        assert_eq!(verifications.executor.account, executor);
        assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
        assert_eq!(verifications.verifier.clone().unwrap().account, verifier);
        assert_eq!(verifications.verifier.clone().unwrap().completed_hash, Some(completed_differing_hash));

    }: _(RawOrigin::Signed(resolver.clone()), task_id, completed_differing_hash)
    verify {
        let task_status = TaskManagementModule::<T>::task_status(task_id).unwrap();
        assert_eq!(task_status, TaskStatusType::Completed);
    }

    impl_benchmark_test_suite!(TaskManagementModule, crate::mock::new_test_ext(), crate::mock::Test);
}