#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

use cyborg_primitives::worker::*;
use frame_support::{sp_runtime::traits::ConstU32, BoundedVec};
use frame_system::RawOrigin;
use scale_info::prelude::vec;

// Define a constant for the worker API domain.
const WORKER_API_DOMAIN1: &str = "https://api-worker1.testing";
const WORKER_API_DOMAIN2: &str = "https://api-worker2.testing";
const WORKER_API_DOMAIN3: &str = "https://api-worker3.testing";
const DOCKER_IMAGE_TESTDATA: &str = "some-docker-imgv.0";

// Helper function to convert the domain string into a BoundedVec with a maximum length of 128 bytes.
// Convert the domain string into a vector of bytes and then into a BoundedVec with a maximum length of 128 bytes.
// The `try_from` mthod ensure that the length of the string doesn't exceed the limit.
fn get_domain(domain_str: &str) -> BoundedVec<u8, ConstU32<128>> {
	BoundedVec::try_from(domain_str.as_bytes().to_vec())
		.expect("Domain string exceeds maximum length")
}

// Helper function to convert task data into a BoundedVec with a maximum length of 128 bytes.
// This ensures the task data string does not exceed the limit of 128 bytes.
fn get_taskdata(task_data_str: &str) -> BoundedVec<u8, ConstU32<128>> {
	BoundedVec::try_from(task_data_str.as_bytes().to_vec())
		.expect("Task Data string exceeds maximum length")
}

/// A function to initialize benchmarking data by creating multiple worker accounts with various attributes
/// for each worker, including location, specifications, and status.
///
/// The function generates 100 creators, and for each creator, it creates up to `MAX_WORKER_ID` workers.
/// It inserts these workers into the `WorkerClusters` map, associating them with their respective creators.
///
/// # Type Parameters
/// - `T`: The configuration trait for the pallet, which provides the necessary types like `AccountId`.
fn set_initial_benchmark_data<T: Config>() {
	for i in 0..100 {
		let index = i;
		let seed = i * 3;
		// Generate the creator account for this worker
		let creator = account::<T::AccountId>("benchmark_account", index, seed);

		// Create the WorkerAPI object
		let api = WorkerAPI {
			domain: match i % 3 {
				1 => get_domain(WORKER_API_DOMAIN1),
				2 => get_domain(WORKER_API_DOMAIN2),
				0 => get_domain(WORKER_API_DOMAIN3),
				_ => get_domain(WORKER_API_DOMAIN1),
			},
		};

		// Create the location object for the worker
		let worker_location = Location {
			latitude: i as i32,
			longitude: (103 + i) as i32,
		};

		// Define the worker's specifications (RAM, storage, CPU)
		let worker_specs = WorkerSpecs {
			ram: 5_000_000_000u64,
			storage: 100_000_000_000u64,
			cpu: 5u16,
		};

		// Maximum number of worker IDs to generate
		const MAX_WORKER_ID: u64 = 6;

		// Insert the MAX_WORKER_ID into the AccountWorkers map for the creator
		pallet_edge_connect::AccountWorkers::<T>::insert(creator.clone(), MAX_WORKER_ID);

		// Loop to create multiple workers for the same creator
		for worker_id in 0..MAX_WORKER_ID {
			// Get the current block number
			let blocknumber = <frame_system::Pallet<T>>::block_number();

			// Create a Worker struct with all the required data
			let worker = Worker {
				id: worker_id,
				owner: creator.clone(),
				location: worker_location.clone(),
				specs: worker_specs.clone(),
				reputation: 0,
				start_block: blocknumber,
				status: WorkerStatusType::Inactive,
				status_last_updated: blocknumber,
				api: api.clone(),
				last_status_check: pallet_timestamp::Pallet::<T>::get(),
			};

			// Insert the worker into the WorkerClusters map
			pallet_edge_connect::WorkerClusters::<T>::insert((creator.clone(), worker_id), worker);

			// Initialize Compute Hours for the creator account in the payment pallet.
			pallet_payment::ComputeHours::<T>::insert(creator.clone(), 50);
		}
	}
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn task_scheduler<T: Config>() -> Result<(), BenchmarkError> {
		// Setup initial benchmark data (this function is assumed to prepopulate necessary state)
		// This function could be used to set up the initial state required for benchmarking.
		set_initial_benchmark_data::<T>();

		// Assign a caller account that will act as the worker's owner.
		let caller: T::AccountId = whitelisted_caller();
		// Create task data.
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);

		let worker_account = account::<T::AccountId>("benchmark_account", 0, 0);

		let worker_id = 0;

		// Initialize Compute Hours for the caller account in the payment pallet.
		// This ensures the account has sufficient compute hours for task operations during benchmarking.
		pallet_payment::ComputeHours::<T>::insert(caller.clone(), 50);

		#[block]
		{
			Pallet::<T>::task_scheduler(
				RawOrigin::Signed(caller.clone()).into(),
				task_data,
				worker_account,
				worker_id,
				Some(10),
			)
			.expect("Failed to schedule task")
		}

		// Verification code
		// We will check the worker's details by looking them up in the worker cluster storage map.
		let task_allocation = TaskAllocations::<T>::iter().collect::<Vec<_>>();
		assert!(task_allocation.len() > 0, "Failed to schedule task");

		let task_owner = TaskOwners::<T>::iter().collect::<Vec<_>>();
		assert!(task_owner.len() > 0, "Failed to schedule task");

		let tasks = Tasks::<T>::iter().collect::<Vec<_>>();
		assert!(tasks.len() > 0, "Failed to schedule task");

		let task_status = TaskStatus::<T>::iter().collect::<Vec<_>>();
		assert!(task_status.len() > 0, "Failed to schedule task");

		Ok(())
	}

	#[benchmark]
	fn submit_completed_task<T: Config>() -> Result<(), BenchmarkError> {
		// Register the executor worker with a domain.
		// This registers an executor worker account and assigns it to a specific domain.
		// The executor will later complete a task.mpleting a task.
		let executor: T::AccountId = account("executor", 0, 0);
		let worker_id = 0;
		let latitude: Latitude = 1;
		let longitude: Longitude = 100;
		let ram: RamBytes = 5_000_000_000u64;
		let storage: StorageBytes = 100_000_000_000u64;
		let cpu: CpuCores = 5u16;
		let domain = get_domain(WORKER_API_DOMAIN1);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(executor.clone()).into(),
			domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Schedule a task for the executor to complete.
		// A caller schedules the task, which will be handled by the registered executor.
		let caller: T::AccountId = whitelisted_caller();
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);
		// Initialize Compute Hours for the caller account in the payment pallet.
		pallet_payment::ComputeHours::<T>::insert(caller.clone(), 50);

		Pallet::<T>::task_scheduler(
			RawOrigin::Signed(caller.clone()).into(),
			task_data.clone(),
			Some(10),
			executor.clone(),
			worker_id,
		)?;

		// Register a verifier worker with a different domain.
		// The verifier is responsible for validating the results of the task completed by the executor.
		let verifier: T::AccountId = account("verifier", 0, 0);
		let domain2 = get_domain(WORKER_API_DOMAIN2);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(verifier.clone()).into(),
			domain2,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Submit the task completion by the executor.
		// The executor submits the completed task's result, along with a result hash.
		let task_id = NextTaskId::<T>::get() - 1;
		let completed_hash = H256([123; 32]);
		let result: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 128]).unwrap();

		// The executor submits the completed task.
		#[block]
		{
			Pallet::<T>::submit_completed_task(
				RawOrigin::Signed(executor.clone()).into(),
				task_id,
				completed_hash.clone(),
				result.clone(),
			)
			.expect("Failed to submit completed task")
		}

		// Verification code
		// Verify the task status and ensure it's pending validation.
		// After submission, the task status should be updated to `PendingValidation`.
		let task_status = TaskStatus::<T>::get(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::PendingValidation);

		// Verify the task's execution and validation state.
		// Check if the executor and verifier information is correctly stored.
		let verifications = Pallet::<T>::get_task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, executor);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
		assert_eq!(verifications.verifier.clone().unwrap().account, verifier);
		assert_eq!(verifications.verifier.clone().unwrap().completed_hash, None);

		Ok(())
	}

	#[benchmark]
	fn verify_completed_task<T: Config>() -> Result<(), BenchmarkError> {
		// Register the executor worker with a domain.
		// This registers an executor worker who will complete the task.
		let executor: T::AccountId = account("executor", 0, 0);
		let worker_id = 0;
		let latitude: Latitude = 1;
		let longitude: Longitude = 100;
		let ram: RamBytes = 5_000_000_000u64;
		let storage: StorageBytes = 100_000_000_000u64;
		let cpu: CpuCores = 5u16;
		let domain = get_domain(WORKER_API_DOMAIN1);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(executor.clone()).into(),
			domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Schedule a task for the executor to complete.
		// A caller schedules the task, which will be completed by the executor.
		let caller: T::AccountId = whitelisted_caller();
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);
		// Initialize Compute Hours for the caller account in the payment pallet.
		pallet_payment::ComputeHours::<T>::insert(caller.clone(), 50);

		Pallet::<T>::task_scheduler(
			RawOrigin::Signed(caller.clone()).into(),
			task_data.clone(),
			Some(10),
			executor.clone(),
			worker_id,
		)?;

		// Register a verifier worker with a different domain.
		// The verifier is responsible for validating the results of the task completed by the executor.
		let verifier: T::AccountId = account("verifier", 0, 0);
		let domain2 = get_domain(WORKER_API_DOMAIN2);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(verifier.clone()).into(),
			domain2,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Submit the task completion by the executor.
		// The executor submits the result of the completed task along with a result hash.
		let task_id = NextTaskId::<T>::get() - 1;
		let completed_hash = H256([123; 32]);
		let result: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 10]).unwrap();
		Pallet::<T>::submit_completed_task(
			RawOrigin::Signed(executor.clone()).into(),
			task_id,
			completed_hash.clone(),
			result.clone(),
		)?;

		// Submit the completed task for verification by the verifier.
		// The verifier validates the task and ensures the result is correct.
		#[block]
		{
			Pallet::<T>::verify_completed_task(
				RawOrigin::Signed(verifier.clone()).into(),
				task_id,
				completed_hash.clone(),
			)
			.expect("Failed to verify completed task")
		}

		// Verify the task status is updated and marked as `Completed`.
		// After verification, the task should have a status of `Completed`.
		let task_status = TaskStatus::<T>::get(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::Completed);

		Ok(())
	}

	#[benchmark]
	fn resolve_completed_task<T: Config>() -> Result<(), BenchmarkError> {
		// Register the executor worker with a domain.
		// This registers an executor worker who will complete the task.
		let executor: T::AccountId = account("executor", 0, 0);
		let worker_id = 0;
		let latitude: Latitude = 1;
		let longitude: Longitude = 100;
		let ram: RamBytes = 5_000_000_000u64;
		let storage: StorageBytes = 100_000_000_000u64;
		let cpu: CpuCores = 5u16;
		let domain = get_domain(WORKER_API_DOMAIN1);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(executor.clone()).into(),
			domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Schedule a task for the executor to complete.
		// A caller schedules the task, which will be completed by the executor.
		let caller: T::AccountId = whitelisted_caller();
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);
		Pallet::<T>::task_scheduler(RawOrigin::Signed(caller.clone()).into(), task_data.clone())?;

		// Register a verifier worker with a different domain.
		// The verifier is responsible for validating the results of the task completed by the executor.
		let verifier: T::AccountId = account("verifier", 0, 0);
		let domain2 = get_domain(WORKER_API_DOMAIN2);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(verifier.clone()).into(),
			domain2,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Submit the task completion by the executor.
		// The executor submits the result of the completed task along with a result hash.
		let task_id = NextTaskId::<T>::get() - 1;
		let completed_hash = H256([4; 32]);
		let result: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 10]).unwrap();
		Pallet::<T>::submit_completed_task(
			RawOrigin::Signed(executor.clone()).into(),
			task_id,
			completed_hash.clone(),
			result.clone(),
		)?;

		// Register the resolver.
		// The resolver is responsible for resolving the task
		let resolver: T::AccountId = account("resolver", 0, 0);
		let domain3 = get_domain(WORKER_API_DOMAIN3);
		pallet_edge_connect::Pallet::<T>::register_worker(
			RawOrigin::Signed(resolver.clone()).into(),
			domain3,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		// Verify completed task by the verifier
		// The verifier checks the task result
		// On the following example, a differing hash to simulate a discrepancy.
		let completed_differing_hash = H256([123; 32]);
		Pallet::<T>::verify_completed_task(
			RawOrigin::Signed(verifier.clone()).into(),
			task_id,
			completed_differing_hash.clone(),
		)?;

		// Check the task verification details.
		// Ensure the executor, verifier, and their respective hashes are correctly stored.
		let verifications = Pallet::<T>::get_task_verifications(task_id).unwrap();
		assert_eq!(verifications.executor.account, executor);
		assert_eq!(verifications.executor.completed_hash, Some(completed_hash));
		assert_eq!(verifications.verifier.clone().unwrap().account, verifier);
		assert_eq!(
			verifications.verifier.clone().unwrap().completed_hash,
			Some(completed_differing_hash)
		);

		// Resolve the task by the resolver.
		// The resolver resolves the task, which concludes the task lifecycle.
		#[block]
		{
			Pallet::<T>::resolve_completed_task(
				RawOrigin::Signed(resolver.clone()).into(),
				task_id,
				completed_differing_hash.clone(),
			)
			.expect("Failed to resolve task")
		}

		// Verify the task status is updated and marked as `Completed`.
		// After the resolver finishes, the task should have a status of `Completed`.
		let task_status = TaskStatus::<T>::get(task_id).unwrap();
		assert_eq!(task_status, TaskStatusType::Completed);

		Ok(())
	}

	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
