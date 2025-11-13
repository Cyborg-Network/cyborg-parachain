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
		// UUIDs for each miner
		// let miner_uuid_cloud = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		// let miner_uuid_edge  = b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec();

		// Full IDs (the pallet adds "CL-" or "ED-" automatically)
		let bounded_uuid_cloud: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		let api_info = MinerAPI { domain: domain.clone() };
		let miner_specs = MinerSpecs { ram, storage, cpu };
		let miner_location = Location {
			latitude,
			longitude,
		};
		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

		let miner_0 = Miner {
			id: bounded_uuid_cloud.clone(),
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
			id: bounded_uuid_edge.clone(),
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
			bounded_uuid_cloud.clone(),
			domain.clone(),
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
			bounded_uuid_edge.clone(),
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(bounded_uuid_cloud),
			Some(miner_0)
		);
		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(bounded_uuid_edge),
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
		// UUIDs for each miner
		// let miner_uuid_cloud = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		// let miner_uuid_edge  = b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec();

		// Full IDs (the pallet adds "CL-" or "ED-" automatically)
		let bounded_uuid_cloud: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		// let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> = 
		// 	b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		let api_info = MinerAPI { domain: domain };
		let miner_specs = MinerSpecs { ram, storage, cpu };
		let miner_location = Location {
			latitude,
			longitude,
		};
		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

		let miner = Miner {
			id: bounded_uuid_cloud.clone(),
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
			bounded_uuid_cloud.clone(),
			api_info.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(bounded_uuid_cloud.clone()),
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

		// UUIDs for each miner
		// let miner_uuid_cloud = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		// let miner_uuid_edge  = b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec();

		// Full IDs (the pallet adds "CL-" or "ED-" automatically)
		let bounded_uuid_cloud: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		let api_info = MinerAPI { domain: domain };

		// Register the first miner
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_0.clone(),
			bounded_uuid_cloud.clone(),
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
				bounded_uuid_cloud.clone(),
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
			bounded_uuid_edge.clone(),
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
				bounded_uuid_edge.clone(),
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
		// UUIDs for each miner
		// let miner_uuid_cloud = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		// let miner_uuid_edge  = b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec();

		// Full IDs (the pallet adds "CL-" or "ED-" automatically)
		let bounded_uuid_cloud: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();
		let api_info = MinerAPI { domain: domain };

		// Register a miner first
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type_0,
			bounded_uuid_cloud.clone(),
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
			bounded_uuid_edge.clone(),
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
			bounded_uuid_cloud.clone()
		));
		// Remove the miner
		assert_ok!(EdgeConnectModule::remove_miner(
			RuntimeOrigin::signed(alice),
			MinerType::Edge,
			bounded_uuid_edge.clone()
		));

		// Assert that the miner no longer exists
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(bounded_uuid_cloud.clone()),
			None
		);
		// Assert that the miner no longer exists
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(bounded_uuid_edge.clone()),
			None
		);
	});
}

#[test]
fn it_fails_for_removing_non_existent_miner() {
	new_test_ext().execute_with(|| {
		let alice = 0;	
		let bounded_uuid_cloud: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();
		// Attempt to remove a miner that doesn't exist
		assert_noop!(
			EdgeConnectModule::remove_miner(RuntimeOrigin::signed(alice), MinerType::Cloud, bounded_uuid_cloud.clone()),
			Error::<Test>::MinerDoesNotExist
		);

		// Attempt to remove a miner that doesn't exist
		assert_noop!(
			EdgeConnectModule::remove_miner(RuntimeOrigin::signed(alice), MinerType::Edge, bounded_uuid_edge.clone()),
			Error::<Test>::MinerDoesNotExist
		);
	});
}

#[test]
fn emiting_proper_event_for_registering_miner() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		// let alice_first_miner_id = 0;
		let domain_str = "foobarkoo.com";
		let domain: BoundedVec<u8, ConstU32<128>> =
			BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap();
		let miner_type = MinerType::Cloud;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;
		// let miner_uuid_cloud = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		// let miner_uuid_edge  = b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec();

		// Full IDs (the pallet adds "CL-" or "ED-" automatically)
		let bounded_uuid_cloud: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		let bounded_uuid_edge: BoundedVec<u8, ConstU32<64>> = 
			b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec().try_into().unwrap();

		System::set_block_number(10);
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type,
			bounded_uuid_cloud.clone(),
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		System::assert_last_event(RuntimeEvent::EdgeConnectModule(Event::MinerRegistered {
			creator: alice,
			miner: (alice, bounded_uuid_cloud),
			domain: domain,
		}));
	})
}

#[test]
fn it_works_for_requesting_maintenance() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let domain_str = "maintenance_test.com";
		let domain: BoundedVec<u8, ConstU32<128>> =
			BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap();
		let miner_type = MinerType::Cloud;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;
		// let miner_uuid = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		let bounded_uuid: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		System::set_block_number(5);

		// Register miner
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type,
			bounded_uuid.clone(),
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Set miner status manually to Busy
		let mut miner = pallet_edge_connect::CloudMiners::<Test>::get(&bounded_uuid).unwrap();
		miner.operational_status = OperationalStatus::Busy;
		pallet_edge_connect::CloudMiners::<Test>::insert(&bounded_uuid, miner.clone());

		assert_ok!(EdgeConnectModule::request_maintenance(
			RuntimeOrigin::signed(alice),
			bounded_uuid.clone(),
			MinerType::Cloud
		));

		let updated = pallet_edge_connect::CloudMiners::<Test>::get(&bounded_uuid).unwrap();
		assert_eq!(updated.operational_status, OperationalStatus::Maintenance);

		assert!(pallet_edge_connect::MinersUnderMaintenance::<Test>::contains_key(&bounded_uuid));

		System::assert_last_event(RuntimeEvent::EdgeConnectModule(Event::MinerUnderMaintenance {
			miner: bounded_uuid.clone(),
			who: alice,
		}));
	});
}

#[test]
fn it_works_for_resolving_maintenance() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let domain_str = "resolve_test.com";
		let domain: BoundedVec<u8, ConstU32<128>> =
			BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap();
		let miner_type = MinerType::Cloud;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;
		// let miner_uuid = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		let bounded_uuid: BoundedVec<u8, ConstU32<64>> = 
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		System::set_block_number(10);

		// Register miner
		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type,
			bounded_uuid.clone(),
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		let mut miner = pallet_edge_connect::CloudMiners::<Test>::get(&bounded_uuid).unwrap();
		miner.operational_status = OperationalStatus::Maintenance;
		pallet_edge_connect::CloudMiners::<Test>::insert(&bounded_uuid, miner.clone());
		pallet_edge_connect::MinersUnderMaintenance::<Test>::insert(&bounded_uuid, 10);
		assert_ok!(EdgeConnectModule::resolve_maintenance(
			RuntimeOrigin::root(),
			bounded_uuid.clone(),
			MinerType::Cloud
		));

		let updated = pallet_edge_connect::CloudMiners::<Test>::get(&bounded_uuid).unwrap();
		assert_eq!(updated.operational_status, OperationalStatus::Available);

		assert!(!pallet_edge_connect::MinersUnderMaintenance::<Test>::contains_key(&bounded_uuid));

		System::assert_last_event(RuntimeEvent::EdgeConnectModule(Event::MaintenanceResolved {
			miner: bounded_uuid.clone(),
		}));
	});
}

#[test]
fn request_maintenance_fails_if_not_owner() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let bob = 1;
		let domain = BoundedVec::try_from(b"ownerfail.com".to_vec()).unwrap();
		let miner_type = MinerType::Cloud;
		// let miner_uuid = b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec();
		let bounded_uuid: BoundedVec<u8, ConstU32<64>> =
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec().try_into().unwrap();

		assert_ok!(EdgeConnectModule::register_miner(
			RuntimeOrigin::signed(alice),
			miner_type,
			bounded_uuid.clone(),
			domain.clone(),
			590000,
			120000,
			100000000,
			100000000,
			12
		));

		let mut miner = pallet_edge_connect::CloudMiners::<Test>::get(&bounded_uuid).unwrap();
		miner.operational_status = OperationalStatus::Busy;
		pallet_edge_connect::CloudMiners::<Test>::insert(&bounded_uuid, miner.clone());

		assert_noop!(
			EdgeConnectModule::request_maintenance(
				RuntimeOrigin::signed(bob),
				bounded_uuid.clone(),
				MinerType::Cloud
			),
			Error::<Test>::NotAuthorized
		);
	});
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
