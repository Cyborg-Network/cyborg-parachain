#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

use cyborg_primitives::{oracle::ProcessStatus, worker::*};
use frame_support::{sp_runtime::traits::ConstU32, BoundedVec};
use frame_system::RawOrigin;
use scale_info::prelude::vec;
use sp_std::vec::Vec;

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

#[benchmarks]
mod benchmarks {
	use super::*;

	//#[benchmark]
	//fn derive_status_percentages_for_period<T: Config>() -> Result<(), BenchmarkError> {
	// Register the executor worker with a domain.
	// This registers an executor worker account and assigns it to a specific domain.
	// The executor will later complete a task.mpleting a task.
	// let executor: T::AccountId = account("executor", 0, 0);
	// let latitude: Latitude = 1;
	// let longitude: Longitude = 100;
	// let ram: RamBytes = 5_000_000_000u64;
	// let storage: StorageBytes = 100_000_000_000u64;
	// let cpu: CpuCores = 5u16;
	//let domain = get_domain(WORKER_API_DOMAIN1);

	// pallet_edge_connect::Pallet::<T>::register_worker(
	// 	RawOrigin::Signed(executor.clone()).into(),
	// 	domain,
	// 	latitude,
	// 	longitude,
	// 	ram,
	// 	storage,
	// 	cpu,
	// )?;
	// Insert the MAX_WORKER_ID into the AccountWorkers map for the creator
	//pallet_edge_connect::AccountWorkers::<T>::insert(executor.clone(), 5);

	// Define the maximum number of feed values to generate for benchmarking.
	//let max_limit = 5;

	// Loop to generate pseudo-random account IDs and worker IDs
	// and simulate inserting data into the system for benchmarking.
	// for seed in 0..max_limit {
	// 	// Generate a pseudo-random account ID using the `account` helper function
	// 	let account_id: T::AccountId = account("benchmark_account", 0, seed);
	// 	// Generate a pseudo-random worker ID
	// 	let worker_id: WorkerId = seed.into();
	// 	// Create a ProcessStatus struct with random online/available status
	// 	let process_status = ProcessStatus {
	// 		online: seed % 2 == 0,
	// 		available: seed % 3 == 0,
	// 	};

	// 	// Insert the generated data into the system for benchmarking purposes.
	// 	Pallet::<T>::on_new_data(
	// 		&account_id.clone(),
	// 		&(account_id.clone(), worker_id),
	// 		&process_status,
	// 	);
	// }

	// Benchmark block to measure performance of `derive_status_percentages_for_period`.
	//#[block]
	//{
	//Pallet::<T>::benchmark_derive_status_percentages_for_period();
	//}

	// Set up a test account and worker ID for validation after data insertion
	// let test_account_id: T::AccountId = account("benchmark_account", 0, 1);
	// let test_worker_id: WorkerId = (1 as u64) * 12345;

	// Assert that WorkerStatusPercentage for the test account and worker ID exists in the
	// ResultingWorkerStatusPercentages storage.
	// assert!(
	// 	ResultingWorkerStatusPercentages::<T>::contains_key((
	// 		test_account_id.clone(),
	// 		test_worker_id
	// 	)),
	// 	"WorkerStatusPercentage not found in storage"
	// );

	// // Assert that WorkerStatus for the test account and worker ID exists in the
	// // ResultingWorkerStatus storage.
	// assert!(
	// 	ResultingWorkerStatus::<T>::contains_key((test_account_id.clone(), test_worker_id)),
	// 	"WorkerStatus not found in storage"
	// );

	//Ok(())
	//}

	#[benchmark]
	fn on_new_data<T: Config>() -> Result<(), BenchmarkError> {
		for i in 0..100 {
			let index = i;
			let seed = i;

			let executor: T::AccountId = account("benchmark_account", index, seed);
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
		}

		#[block]
		{
			let max_limit = 10;

			// Loop to generate and insert mock data for benchmarking
			for i in 0..max_limit {
				let index = i;
				let seed = i;
				// Generate a pseudo-random account ID using the `account` helper function
				let account_id: T::AccountId = account("benchmark_account", index, seed);
				// Generate a pseudo-random worker ID
				let worker_id: WorkerId = seed.into();
				// Create a ProcessStatus struct with random online/available status
				let process_status = ProcessStatus {
					online: i % 2 == 0,
					available: i % 3 == 0,
				};

				// Call the `on_new_data` function of the pallet with generated data
				Pallet::<T>::on_new_data(
					&account_id.clone(),
					&(account_id.clone(), worker_id),
					&process_status,
				);
			}
		}

		// Verification code
		// After submit new data, the submitted per period is log

		let submittedperperiod: Vec<_> = SubmittedPerPeriod::<T>::iter().collect::<Vec<_>>();
		assert!(submittedperperiod.len() > 0, "Submission not found");


		// 	// Set up a test account and worker ID for validation after data insertion
		// 	let test_account_id: T::AccountId = account("benchmark_account", 0, 1);
		// 	let test_worker_id: WorkerId = 1;

		// 	// Assert that submission exists for the given account and worker ID in SubmittedPerPeriod

		// 	let task_allocation: Vec<((T::AccountId, (T::AccountId, WorkerId)), bool)> = SubmittedPerPeriod::<T>::iter().collect::<Vec<_>>();

		//println!( "DEBUG {:1}", task_allocation.len());
		//assert!(task_allocation.len() > 0, "Failed to schedule task");

		// assert!(
		// 	SubmittedPerPeriod::<T>::get((
		// 		test_account_id.clone(),
		// 		(test_account_id.clone(), test_worker_id)
		// 	)),

		// 	"Submission not found"
		// );

		// 	// Assert that key exists in WorkerStatusEntriesPerPeriod for the test account and worker ID
		// 	assert!(
		// 		WorkerStatusEntriesPerPeriod::<T>::contains_key((test_account_id.clone(), test_worker_id)),
		// 		"Entry key does not exists in WorkerStatusEntriesPerPeriod"
		// 	);

		Ok(())
	}

	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
