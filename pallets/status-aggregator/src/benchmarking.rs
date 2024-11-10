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

	/// Benchmark for the `on_new_data` function, which simulates inserting new worker data into the system.
	/// The function generates 100 workers, registers them using the `register_worker` function, and then
	/// calls the `on_new_data` function to simulate the process of feeding data into the system.
	/// It also includes verification steps to ensure the data has been correctly stored.
	#[benchmark]
	fn on_new_data<T: Config>() -> Result<(), BenchmarkError> {
		// Loop to create 100 workers, register them, and prepare for benchmarking.
		for i in 0..100 {
			let index = i;
			let seed = i;

			// Generate a worker account (executor) using the `account` helper function.
			let executor: T::AccountId = account("benchmark_account", index, seed);
			let latitude: Latitude = 1;
			let longitude: Longitude = 100;
			let ram: RamBytes = 5_000_000_000u64;
			let storage: StorageBytes = 100_000_000_000u64;
			let cpu: CpuCores = 5u16;
			let domain = get_domain(WORKER_API_DOMAIN1);

			// Register the worker by calling `register_worker` from the pallet_edge_connect pallet.
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

		// Block for benchmarking the actual data submission.
		let max_limit = 10;

		#[block]
		{
			// Loop to generate and insert mock worker status data into the system for benchmarking.
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

				// Call the `on_new_data` function of the pallet with the generated data.
				Pallet::<T>::on_new_data(
					&account_id.clone(),
					&(account_id.clone(), worker_id),
					&process_status,
				);
			}
		}

		// Verification code to ensure data has been correctly stored.

		// Check that submissions exist in `SubmittedPerPeriod` storage after data is submitted.
		let submittedperperiod: Vec<_> = SubmittedPerPeriod::<T>::iter().collect::<Vec<_>>();
		assert!(submittedperperiod.len() > 0, "Submission not found");

		// Check that entries exist in `WorkerStatusEntriesPerPeriod` storage after data is submitted.
		let workstatusentriesperperiod: Vec<_> =
			WorkerStatusEntriesPerPeriod::<T>::iter().collect::<Vec<_>>();
		assert!(
			workstatusentriesperperiod.len() > 0,
			"WorkerStatusEntriesPerPeriod not found"
		);

		Ok(())
	}

	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
