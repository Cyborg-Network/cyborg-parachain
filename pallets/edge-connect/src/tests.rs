use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, sp_runtime::traits::ConstU32, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;

use cyborg_primitives::miner::*;
use sp_std::convert::TryFrom;

#[test]
fn it_works_for_inserting_miner_into_correct_storage() {
	new_test_ext().execute_with(|| {
		let domain_str = "some_api_domain.com";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let miner_type_0 = MinerType::Cloud;
		let miner_type_1 = MinerType::Edge;
		let latitude: Latitude = 590000;
		let current_task = None;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		let alice = 0;
		let api_info = MinerAPI {
			domain: domain.clone(),
		};
		let miner_specs = MinerSpecs { ram, storage, cpu };
		let miner_location = Location {
			latitude,
			longitude,
		};
		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

		let miner_0 = Miner {
			id: 0,
			owner: alice,
			start_block: 10,
			oracle_status: OracleStatus::Offline,
			operational_status: OperationalStatus::Available,
			status_last_updated: 10,
			current_task: current_task.clone(),
			api: api_info.clone(),
			location: miner_location.clone(),
			specs: miner_specs.clone(),
			reputation: MinerReputation::<BlockNumberFor<Test>>::default(),
			last_status_check: current_timestamp,
		};

		let miner_1 = Miner {
			id: 1,
			owner: alice,
			start_block: 10,
			oracle_status: OracleStatus::Offline,
			operational_status: OperationalStatus::Available,
			status_last_updated: 10,
			current_task: current_task.clone(),
			api: api_info.clone(),
			location: miner_location.clone(),
			specs: miner_specs.clone(),
			reputation: MinerReputation::<BlockNumberFor<Test>>::default(),
			last_status_check: current_timestamp,
		};

		// Dispatch a signed extrinsic.
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_0,
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Dispatch a signed extrinsic.
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_1,
			api_info.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get((alice, 0)),
			Some(miner_0)
		);
		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get((alice, 1)),
			Some(miner_1)
		);
	});
}

#[test]
fn it_works_for_registering_domain() {
	new_test_ext().execute_with(|| {
		let domain_str = "some_api_domain.com";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let miner_type = MinerType::Cloud;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let current_task = None;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		let alice = 0;
		let api_info = MinerAPI {
			domain: domain.clone(),
		};
		let miner_specs = MinerSpecs { ram, storage, cpu };
		let miner_location = Location {
			latitude,
			longitude,
		};
		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

		let miner = Miner {
			id: 0,
			owner: alice,
			start_block: 10,
			oracle_status: OracleStatus::Offline,
			operational_status: OperationalStatus::Available,
			status_last_updated: 10,
			current_task: current_task.clone(),
			api: api_info.clone(),
			location: miner_location.clone(),
			specs: miner_specs.clone(),
			reputation: MinerReputation::<BlockNumberFor<Test>>::default(),
			last_status_check: current_timestamp,
		};

		// Dispatch a signed extrinsic.
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type,
			api_info.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get((alice, 0)),
			Some(miner)
		);
	});
}

#[test]
fn it_fails_for_registering_duplicate_miner() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		let domain_str = "127.0.0.1:3001";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let miner_type_0 = MinerType::Cloud;
		let miner_type_1 = MinerType::Edge;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		let api_info = MinerAPI { domain: domain };

		// Register the first miner
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_0.clone(),
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Try to register the same miner again
		assert_noop!(
			EdgeConnectModule::register_miner(
				RuntimeOrigin::signed(alice),
				miner_type_0,
				api_info.domain.clone(),
				latitude,
				longitude,
				ram,
				storage,
				cpu
			),
			Error::<Test>::MinerExists
		);

		// Register the first miner
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_1.clone(),
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Try to register the same miner again
		assert_noop!(
			EdgeConnectModule::register_miner(
				RuntimeOrigin::signed(alice),
				miner_type_1,
				api_info.domain,
				latitude,
				longitude,
				ram,
				storage,
				cpu
			),
			Error::<Test>::MinerExists
		);
	});
}

#[test]
fn it_works_for_removing_miner() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		let domain_str = "127.0.0.1:3001";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let miner_type_0 = MinerType::Cloud;
		let miner_type_1 = MinerType::Edge;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		let api_info = MinerAPI { domain: domain };

		// Register a miner first
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_0,
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Register a miner first
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_1,
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Remove the miner
		assert_ok!(EdgeConnectModule::remove_miner(
			RuntimeOrigin::signed(alice),
			MinerType::Cloud,
			0
		));
		// Remove the miner
		assert_ok!(EdgeConnectModule::remove_miner(
			RuntimeOrigin::signed(alice),
			MinerType::Edge,
			1
		));

		// Assert that the miner no longer exists
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get((alice, 0)),
			None
		);
		// Assert that the miner no longer exists
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get((alice, 1)),
			None
		);
	});
}

#[test]
fn it_fails_for_removing_non_existent_miner() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		// Attempt to remove a miner that doesn't exist
		assert_noop!(
			EdgeConnectModule::remove_miner(RuntimeOrigin::signed(alice), MinerType::Cloud, 0),
			Error::<Test>::MinerDoesNotExist
		);

		// Attempt to remove a miner that doesn't exist
		assert_noop!(
			EdgeConnectModule::remove_miner(RuntimeOrigin::signed(alice), MinerType::Edge, 0),
			Error::<Test>::MinerDoesNotExist
		);
	});
}

#[test]
fn emiting_proper_event_for_registering_miner() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let alice_first_miner_id = 0;
		let domain_str = "foobarkoo.com";
		let domain: BoundedVec<u8, ConstU32<128>> =
			BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap();
		let miner_type = MinerType::Cloud;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type,
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		System::assert_last_event(RuntimeEvent::EdgeConnectModule(Event::MinerRegistered {
			creator: alice,
			miner: (alice, alice_first_miner_id),
			domain: domain,
		}));
	})
}

/*

	let domain_str = "some_api_domain.com";
	let domain_vec = domain_str.as_bytes().to_vec();
	let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();

	System::set_block_number(10);
	let alice = 0;
	let api_info = MinerAPI { domain: domain };

	let miner = Miner {
		id: 0,
		owner: alice,
		start_block: 10,
		status: MinerStatusType::Inactive,
		api: api_info.clone(),
	};

	// Dispatch a signed extrinsic.
	assert_ok!(EdgeConnectModule::register_miner(
		RuntimeOrigin::signed(alice),
		api_info.domain
	));
	// Read pallet storage and assert an expected result.
	assert_eq!(
		EdgeConnectModule::get_miner_clusters((alice, 0)),
		Some(miner)
	);

*/
