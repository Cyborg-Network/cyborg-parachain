//! Benchmarking setup for pallet-task-management
use super::*;
use crate::{Config, Pallet as TaskManagementModule};
use cyborg_primitives::worker::WorkerAPI;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_core::H256;

// Define a constant for the worker API domain.
const WORKER_API_DOMAIN: &str = "https://api-worker.testing";
const WORKER_API_DOMAIN2: &str = "https://api-worker2.testing";
const WORKER_API_DOMAIN3: &str = "https://api-worker3.testing";
const DOCKER_IMAGE_TEST_DATA: &str = "some-docker-imgv.0";

// Helper function to convert the domain string into a BoundedVec with a maximum length of 128 bytes.
// Convert the domain string into a vector of bytes and then into a BoundedVec with a maximum length of 128 bytes.
// The `try_from` mthod ensure that the length of the string doesn't exceed the limit.
fn get_domain(domain_str: &str) -> BoundedVec<u8, ConstU32<128>> {
	BoundedVec::try_from(domain_str.as_bytes().to_vec())
		.expect("Domain string exceeds maximum length")
}

// Helper function to convert task data into a BoundedVec with a maximum length of 128 bytes.
// This ensures the task data string does not exceed the limit of 128 bytes.
fn get_task_data(task_data_str: &str) -> BoundedVec<u8, ConstU32<128>> {
	BoundedVec::try_from(task_data_str.as_bytes().to_vec())
		.expect("Task Data string exceeds maximum length")
}

benchmarks! {
		// Benchmark for scheduling a task
		task_scheduler {
				let s in 0 .. 100;

				// Retrieve a whitelisted account to use as the caller (sender of the transaction)
				// This is a pre-authorized account used for benchmarking purposes.
				let caller: T::AccountId = whitelisted_caller();

				// Get the domain for the worker.
				let domain=get_domain(WORKER_API_DOMAIN);

				// Register a worker for the caller with the specified domain.
				let api_info = WorkerAPI {
						domain,
				};

				// Register a worker for the caller with the specified domain.
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(caller.clone()).into(), api_info.domain)?;

				// Create task data.
				let task_data=get_task_data(DOCKER_IMAGE_TEST_DATA);

				// Benchmark the task scheduling
		}: _(RawOrigin::Signed(caller.clone()), task_data.clone())
		verify {
					// Verify that the task has been correctly scheduled.
				let task_id = TaskManagementModule::<T>::next_task_id() - 1;
				let task_info = TaskManagementModule::<T>::get_tasks(task_id).unwrap();
				assert_eq!(task_info.metadata, task_data);
				assert_eq!(task_info.task_owner, caller);
		}

		// Benchmark for submitting a completed task
		submit_completed_task {
				let s in 0 .. 100;

				// Retrieve a whitelisted account to use as the caller (sender of the transaction)
				// This is a pre-authorized account used for benchmarking purposes.
				// Retrieve the caller and register the executor.
				let caller: T::AccountId = whitelisted_caller();
				let executor: T::AccountId = account("executor", 0, 0);

				// Register the executor worker with a domain.
				let domain=get_domain(WORKER_API_DOMAIN);
				let api_info = WorkerAPI  {
						domain,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(executor.clone()).into(), api_info.domain)?;

				// Create task data and schedule the task.
				let task_data = get_task_data(DOCKER_IMAGE_TEST_DATA);
				TaskManagementModule::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;

				// Register the verifier with a different domain.
				let verifier: T::AccountId = account("verifier", 0, 0);
				let domain2=get_domain(WORKER_API_DOMAIN2);
				let api_info_verifier = WorkerAPI  {
						domain:domain2,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(verifier.clone()).into(), api_info_verifier.domain)?;

				// Complete the task by the executor.
				let task_id = TaskManagementModule::<T>::next_task_id() - 1;
				let completed_hash = H256([123; 32]);

		// Benchmark the submission of a completed task
		}: _(RawOrigin::Signed(executor.clone()), task_id, completed_hash)
		verify {
				// Verify that the task status is updated and the completed hash is recorded.
				let task_status = TaskManagementModule::<T>::task_status(task_id).unwrap();
				assert_eq!(task_status, TaskStatusType::PendingValidation);

				let verifications = TaskManagementModule::<T>::task_verifications(task_id).unwrap();
				assert_eq!(verifications.executor.account, executor);
				assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
				assert_eq!(verifications.verifier.clone().unwrap().account, verifier);
				assert_eq!(verifications.verifier.clone().unwrap().completed_hash, None);
		}

		// Benchmark for verifying a completed task
		verify_completed_task {
				let s in 0 .. 100;

				// Retrieve a whitelisted account to use as the caller (sender of the transaction)
				// This is a pre-authorized account used for benchmarking purposes.
				let caller: T::AccountId = whitelisted_caller();
				let executor: T::AccountId = account("executor", 0, 0);
				let verifier: T::AccountId = account("verifier", 0, 0);

				// Register the executor worker and schedule the task.
				let domain=get_domain(WORKER_API_DOMAIN);
				let api_info_executor = WorkerAPI {
						domain,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(executor.clone()).into(), api_info_executor.domain)?;

				// Create a task data
				let task_data = get_task_data(DOCKER_IMAGE_TEST_DATA);
				TaskManagementModule::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;

				// Register the verifier and complete the task by the executor.
				let domain2=get_domain(WORKER_API_DOMAIN2);

				let api_info_verifier = WorkerAPI  {
						domain:domain2,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(verifier.clone()).into(), api_info_verifier.domain)?;

				let task_id = TaskManagementModule::<T>::next_task_id() - 1;

				let completed_hash = H256([123; 32]);
				TaskManagementModule::<T>::submit_completed_task(RawOrigin::Signed(executor.clone()).into(), task_id, completed_hash)?;

		// Benchmark the verification of a completed task
		}: _(RawOrigin::Signed(verifier.clone()), task_id, completed_hash)
		verify {
				// Verify the task status is updated and marked as completed.
				let task_status = TaskManagementModule::<T>::task_status(task_id).unwrap();
				assert_eq!(task_status, TaskStatusType::Completed);
		}

		// Benchmark for resolving a completed task by the resolver
		resolve_completed_task {
				let s in 0 .. 100;

				// Retrieve a whitelisted account to use as the caller (sender of the transaction)
				// This is a pre-authorized account used for benchmarking purposes.
				let caller: T::AccountId = whitelisted_caller();

				let executor: T::AccountId = account("executor", 0, 0);
				let verifier: T::AccountId = account("verifier", 0, 0);
				let resolver: T::AccountId = account("resolver", 0, 0);

				// Register workers for the executor, verifier, and resolver.
				let domain=get_domain(WORKER_API_DOMAIN);
				let api_info_executor = WorkerAPI {
						domain,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(executor.clone()).into(), api_info_executor.domain)?;

				// Schedule a task.
				let task_data = get_task_data(DOCKER_IMAGE_TEST_DATA);
				TaskManagementModule::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;
				let task_id = TaskManagementModule::<T>::next_task_id() - 1;

				// Register verifier and submit completed task by the executor.
				let domain2=get_domain(WORKER_API_DOMAIN2);

				let api_info_verifier = WorkerAPI {
						domain:domain2,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(verifier.clone()).into(), api_info_verifier.domain)?;

				// Submit completed task by the executor
				let completed_hash = H256([4; 32]);
				TaskManagementModule::<T>::submit_completed_task(RawOrigin::Signed(executor.clone()).into(), task_id, completed_hash)?;

				// Register the resolver.
				let domain3=get_domain(WORKER_API_DOMAIN3);

				let api_info_resolver = WorkerAPI {
						domain:domain3,
				};
				pallet_edge_connect::Pallet::<T>::register_worker(RawOrigin::Signed(resolver.clone()).into(), api_info_resolver.domain)?;

				let completed_differing_hash = H256([123; 32]);
				// Verify completed task by the verifier
				TaskManagementModule::<T>::verify_completed_task(RawOrigin::Signed(verifier.clone()).into(), task_id, completed_differing_hash)?;

				let verifications = TaskManagementModule::<T>::task_verifications(task_id).unwrap();
				assert_eq!(verifications.executor.account, executor);
				assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
				assert_eq!(verifications.verifier.clone().unwrap().account, verifier);
				assert_eq!(verifications.verifier.clone().unwrap().completed_hash, Some(completed_differing_hash));

		// Benchmark the resolution of the task by the resolver
		}: _(RawOrigin::Signed(resolver.clone()), task_id, completed_differing_hash)
		verify {
				// Verify the task status is updated to completed.
				let task_status = TaskManagementModule::<T>::task_status(task_id).unwrap();
				assert_eq!(task_status, TaskStatusType::Completed);
		}

		impl_benchmark_test_suite!(TaskManagementModule, crate::mock::new_test_ext(), crate::mock::Test);
}
