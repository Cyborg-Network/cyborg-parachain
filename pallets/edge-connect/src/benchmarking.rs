#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

use cyborg_primitives::worker::*;
use frame_support::{sp_runtime::traits::ConstU32, BoundedVec};
use frame_system::RawOrigin;

// Define a constant for the worker API domain.
const WORKER_API_DOMAIN: &str = "https://some_api_domain.com";

// Helper function to convert the domain string into a BoundedVec with a maximum length of 128 bytes.
// Convert the domain string into a vector of bytes and then into a BoundedVec with a maximum length of 128 bytes.
// The `try_from` mthod ensure that the length of the string doesn't exceed the limit.
fn get_domain(domain_str: &str) -> BoundedVec<u8, ConstU32<128>> {
	BoundedVec::try_from(domain_str.as_bytes().to_vec())
		.expect("Domain string exceeds maximum length")
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
			domain: get_domain(WORKER_API_DOMAIN),
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
		AccountWorkers::<T>::insert(creator.clone(), MAX_WORKER_ID);

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
			WorkerClusters::<T>::insert((creator.clone(), worker_id), worker);
		}
	}
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_worker<T: Config>() -> Result<(), BenchmarkError> {
		// Setup initial benchmark data (this function is assumed to prepopulate necessary state)
		// This function could be used to set up the initial state required for benchmarking.
		set_initial_benchmark_data::<T>();

		// Define the caller (account) that will be used to register the worker.
		// `whitelisted_caller()` provides a benchmark-friendly account for testing.
		let caller: T::AccountId = whitelisted_caller();

		// Define the worker's domain and specifications for registration.
		let domain = get_domain(WORKER_API_DOMAIN);
		let latitude = 1i32;
		let longitude = 103i32;
		let ram = 5_000_000_000u64;
		let storage = 100_000_000_000u64;
		let cpu = 8u16;

		// Benchmark the execution of registering the worker.
		// The `#[block]` attribute indicates that the following block of code is being benchmarked.
		#[block]
		{
			Pallet::<T>::register_worker(
				RawOrigin::Signed(caller.clone()).into(),
				domain,
				latitude,
				longitude,
				ram,
				storage,
				cpu,
			)?;
		}

		// Verification code
		// We will check the worker's details by looking them up in the worker cluster storage map.
		let worker_id = 0;

		// Match the result of querying the worker from the worker cluster storage.
		// The key is (T::AccountId, WorkerId), where `caller` is the account and `worker_id` is the ID.
		let worker_id = 0;
		match WorkerClusters::<T>::get((caller.clone(), worker_id)) {
			Some(worker) => {
				let id = worker.id;
				let owner = worker.owner;
				let location = worker.location;
				let specs = worker.specs;

				// Verification: Ensure that the worker's ID matches the expected worker_id.
				assert!(id == worker_id)
			}
			None => {
				// If the worker is not found, we panic because it indicates that the worker was not registered properly.
				panic!("Worker not found!")
			}
		}

		Ok(())
	}

	#[benchmark]
	fn remove_worker<T: Config>() -> Result<(), BenchmarkError> {
		// Setup initial benchmark data (this function is assumed to prepopulate necessary state)
		set_initial_benchmark_data::<T>();

		// Assign a caller account that will act as the worker's owner.
		let caller: T::AccountId = whitelisted_caller();

		// Define worker's location and specifications.
		let domain = get_domain(WORKER_API_DOMAIN);
		let latitude = 1i32;
		let longitude = 103i32;
		let ram = 5_000_000_000u64;
		let storage = 100_000_000_000u64;
		let cpu = 8u16;

		// Register the worker under the specified caller with the above configurations.
		Pallet::<T>::register_worker(
			RawOrigin::Signed(caller.clone()).into(),
			domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu,
		)?;

		let worker_id = 0;

		// Benchmark the execution of removing the worker (block of code to measure).
		#[block]
		{
			Pallet::<T>::remove_worker(RawOrigin::Signed(caller.clone()).into(), worker_id)?;
		}

		// Verification code:
		// Ensure that the worker has been removed from the storage map.
		// Key used: (T::AccountId, WorkerId)
		// Retrieve the worker cluster to check if the worker is still present.
		let worker_cluster = WorkerClusters::<T>::get((caller.clone(), worker_id));

		// Assert that the worker is no longer in the cluster (i.e., must be `None` after deletion).
		assert!(worker_cluster.is_none(), "Worker not deleted");

		Ok(())
	}
	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
