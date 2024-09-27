#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

use cyborg_primitives::worker::*;
use frame_support::{sp_runtime::traits::ConstU32, BoundedVec};
use frame_system::RawOrigin;

//use crate::{Config, Pallet as TaskManagementModule};
//use cyborg_primitives::worker::WorkerAPI;
//use frame_benchmarking::{account, benchmarks, whitelisted_caller};
//use frame_support::BoundedVec;
//use frame_system::RawOrigin;
//use scale_info::prelude::vec;
//use sp_core::H256;

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

		// Create a signed origin for the caller, required for dispatchable function calls.
		let origin = RawOrigin::Signed(caller.clone());

		// Create task data.
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);

		#[block]
		{
			Pallet::<T>::task_scheduler(origin.into(), task_data).expect("Failed to schedule task")
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
		// Setup initial benchmark data (this function is assumed to prepopulate necessary state)
		// This function could be used to set up the initial state required for benchmarking.
		set_initial_benchmark_data::<T>();

	
		let caller: T::AccountId = whitelisted_caller();
		let origin = RawOrigin::Signed(caller.clone());
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);		
		Pallet::<T>::task_scheduler(origin.into(), task_data).expect("Failed to schedule task");


		let origin = RawOrigin::Signed(caller.clone());
		let task_id = Pallet::<T>::next_task_id();

		let next_task = NextTaskId::<T>::get();
		
		println!("next_task {:1}", next_task);
		let completed_hash = H256([123; 32]);
		let result: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 10]).unwrap();


		#[block]
		{
			//Pallet::<T>::submit_completed_task(origin.into(),task_id, completed_hash.clone(), result.clone()).expect("Failed to submit completed task")
		}

		//TaskVerifications::<T>::insert(task_id, ver.clone());

		Ok(())
	}

	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

/*
benchmarks! {
		// Benchmark for scheduling a task
		


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
				let dummy_bounded_vec: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 10]).unwrap();
		
		// Benchmark the submission of a completed task
		}: _(RawOrigin::Signed(executor.clone()), task_id, completed_hash,dummy_bounded_vec)
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
				let dummy_bounded_vec: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 10]).unwrap();

				TaskManagementModule::<T>::submit_completed_task(RawOrigin::Signed(executor.clone()).into(), task_id, completed_hash, dummy_bounded_vec)?;

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
				let dummy_bounded_vec: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(vec![0u8; 10]).unwrap();
				TaskManagementModule::<T>::submit_completed_task(RawOrigin::Signed(executor.clone()).into(), task_id, completed_hash,dummy_bounded_vec)?;

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

		*/
