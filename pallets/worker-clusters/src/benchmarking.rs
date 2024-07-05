//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as WorkerClustersPallet;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::sp_runtime::traits::One;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_worker() {
		let caller: T::AccountId = whitelisted_caller();

		let api_info = WorkerAPI {
			ip: None,
			domain: Some(100)
		};

		let worker = Worker {
			id: 0,
			owner: caller.clone(),
			start_block: One::one(),
			status: WorkerStatusType::Inactive,
			api: api_info.clone()
		};

		#[extrinsic_call]
		register_worker(RawOrigin::Signed(caller.clone()), api_info.ip, api_info.domain);

		assert_eq!(WorkerClustersPallet::<T>::get_worker_clusters((caller.clone(),0)), Some(worker));
	}

	#[benchmark]
	fn remove_worker() {
		let caller: T::AccountId = whitelisted_caller();

		// First, register a worker to ensure there is one to remove
		let api_info = WorkerAPI {
			ip: ip: Some(Ip { ipv4: Some(125), ipv6: None, port: 123}),
			domain: None
		};

		let worker = Worker {
			id: 0,
			owner: caller.clone(),
			start_block: <frame_system::Pallet<T>>::block_number(),
			status: WorkerStatusType::Inactive,
			api: api_info.clone()
		};

		WorkerClustersPallet::<T>::register_worker(
			RawOrigin::Signed(caller.clone()).into(),
			api_info.ip,
			api_info.domain,
		).unwrap();

		#[extrinsic_call]
		remove_worker(RawOrigin::Signed(caller.clone()), 0);

		assert_eq!(WorkerClustersPallet::<T>::get_worker_clusters((caller.clone(), 0)), None);
	}

	impl_benchmark_test_suite!(WorkerClustersPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
