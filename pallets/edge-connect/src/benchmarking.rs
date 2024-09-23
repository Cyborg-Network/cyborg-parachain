//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as edgeConnectPallet;
use frame_benchmarking::v2::*;
use frame_support::sp_runtime::traits::ConstU32;
use frame_support::sp_runtime::traits::One;
use frame_support::BoundedVec;
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

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_worker() {
		// Retrieve a whitelisted account to use as the caller (sender of the transaction)
		// This is a pre-authorized account used for benchmarking purposes.
		let caller: T::AccountId = whitelisted_caller();

		// Define a domain string that will be part of the WorkerAPI (represent the worker's domain).
		let domain = get_domain(WORKER_API_DOMAIN);

		// Create a WorkerAPI struct that contains the domain information.
		let api_info = WorkerAPI {
			domain, // Store the domain in the API Info
		};

		// Create a Worker struct with the relevant details.
		// The worker is assigned an ID of 0, owned by the caller, start at block 1, and  is initially inactive.
		// The worker's API information includes the domain created above.
		let worker = Worker {
			id: 0,
			owner: caller.clone(),
			start_block: One::one(),
			status: WorkerStatusType::Inactive,
			api: api_info.clone(),
			status_last_updated: One::one(),
		};

		// Call the `register_worker` extrinsic using the #[extrinsic_call] macro, which will benchmark the extrinsic.
		// The `RawOrigin::Signed(caller.clone())` indicates that this transaction is signed by the caller.
		// The domain  from `api_info` is passed as part of the worker registration process.
		#[extrinsic_call]
		register_worker(RawOrigin::Signed(caller.clone()), api_info.domain);

		// After the worker is registered, assert that the worker was correctly stored in the pallet's storage.
		// `get_worker_cluster` (getter fn for workercluster) retrieves the worker by the tuple (caller, worker ID).
		// If the worker exists in storage, the assertion passes, indicating that the registration succeeded.
		assert_eq!(
			edgeConnectPallet::<T>::get_worker_clusters((caller.clone(), 0)),
			Some(worker)
		);
	}

	#[benchmark]
	fn remove_worker() {
		// Retrieve a whitelisted account to use as the caller (sender of the transaction)
		// This is a pre-authorized account used for benchmarking purposes.
		let caller: T::AccountId = whitelisted_caller();

		// Define a domain string that will be part of the WorkerAPI (represent the worker's domain).
		let domain = get_domain(WORKER_API_DOMAIN);

		// First, register a worker to ensure there is one to remove
		// This step simulates the existence of a worker in the system before we benchmark the removal process.
		let api_info = WorkerAPI { domain };

		// Call the `register_worker` extrinsic to create and register a worker.
		// This step is necessary before calling `remove_worker` because we need to remove an existing worker.
		edgeConnectPallet::<T>::register_worker(
			RawOrigin::Signed(caller.clone()).into(),
			api_info.domain,
		)
		.unwrap();

		// Call the `remove_worker` extrinsic using the #[extrinsic_call] macro, which will benchmark the extrinsic.
		// the `RawOrigin::Signed(caller.clone())` indicates that this transaction is signed by the caller.
		// Remove the worker with ID 0
		#[extrinsic_call]
		remove_worker(RawOrigin::Signed(caller.clone()), 0);

		// After the `remove_worker` extrinsic is called, assert that the worker has been succesfully removed from storage.
		// `get_worker_cluster` (getter fn for workercluster) should return `None` for the worker with ID 0, indicating the worker no longer exists.
		assert_eq!(
			edgeConnectPallet::<T>::get_worker_clusters((caller.clone(), 0)),
			None
		);
	}

	impl_benchmark_test_suite!(
		edgeConnectPallet,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
