use crate::{mock::*, Error, Event};
pub use cyborg_primitives::worker::*;

use frame_support::sp_runtime::traits::ConstU32;
use frame_support::BoundedVec;
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryFrom;

// #[test]
// fn it_works_for_registering_domain() {
// 	new_test_ext().execute_with(|| {
// 		let domain_str = "some_api_domain.com";
// 		let domain_vec = domain_str.as_bytes().to_vec();
// 		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
// 		let latitude: Latitude = 590000;
// 		let longitude: Longitude = 120000;
// 		let ram: RamBytes = 100000000;
// 		let storage: StorageBytes = 100000000;
// 		let cpu: CpuCores = 12;

// 		System::set_block_number(10);
// 		let alice = 0;
// 		let api_info = WorkerAPI { domain: domain };
// 		let worker_specs = WorkerSpecs { ram, storage, cpu };
// 		let worker_location = Location {
// 			latitude,
// 			longitude,
// 		};
// 		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

// 		let worker = Worker {
// 			id: 0,
// 			owner: alice,
// 			start_block: 10,
// 			status: WorkerStatusType::Inactive,
// 			status_last_updated: 10,
// 			api: api_info.clone(),
// 			location: worker_location.clone(),
// 			specs: worker_specs.clone(),
// 			reputation: 0,
// 			last_status_check: current_timestamp,
// 		};

// 		// Dispatch a signed extrinsic.
// 		assert_ok!(edgeConnectModule::register_worker(
// 			RuntimeOrigin::signed(alice),
// 			api_info.domain,
// 			latitude,
// 			longitude,
// 			ram,
// 			storage,
// 			cpu
// 		));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(
// 			edgeConnectModule::get_worker_clusters((alice, 0)),
// 			Some(worker)
// 		);
// 	});
// }
