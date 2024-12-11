use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, sp_runtime::traits::ConstU32, BoundedVec};

use cyborg_primitives::worker::*;
use sp_std::convert::TryFrom;

#[test]
fn it_works_for_inserting_worker_into_correct_storage() {
	new_test_ext().execute_with(|| {
		let domain_str = "some_api_domain.com";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let worker_type_0 = WorkerType::Docker;
		let worker_type_1 = WorkerType::Executable;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		let alice = 0;
		let api_info = WorkerAPI { domain: domain };
		let worker_specs = WorkerSpecs { ram, storage, cpu };
		let worker_location = Location {
			latitude,
			longitude,
		};
		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

		let worker_0 = Worker {
			id: 0,
			owner: alice,
			start_block: 10,
			status: WorkerStatusType::Inactive,
			status_last_updated: 10,
			api: api_info.clone(),
			location: worker_location.clone(),
			specs: worker_specs.clone(),
			reputation: 0,
			last_status_check: current_timestamp,
		};

		let worker_1 = Worker {
			id: 1,
			owner: alice,
			start_block: 10,
			status: WorkerStatusType::Inactive,
			status_last_updated: 10,
			api: api_info.clone(),
			location: worker_location.clone(),
			specs: worker_specs.clone(),
			reputation: 0,
			last_status_check: current_timestamp,
		};

		// Dispatch a signed extrinsic.
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_0,
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Dispatch a signed extrinsic.
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_1,
			api_info.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::WorkerClusters::<Test>::get((alice, 0)),
			Some(worker_0)
		);
		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::ExecutableWorkers::<Test>::get((alice, 1)),
			Some(worker_1)
		);
	});
}

#[test]
fn it_works_for_registering_domain() {
	new_test_ext().execute_with(|| {
		let domain_str = "some_api_domain.com";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let worker_type = WorkerType::Docker;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		let alice = 0;
		let api_info = WorkerAPI { domain: domain };
		let worker_specs = WorkerSpecs { ram, storage, cpu };
		let worker_location = Location {
			latitude,
			longitude,
		};
		let current_timestamp = pallet_timestamp::Pallet::<Test>::get();

		let worker = Worker {
			id: 0,
			owner: alice,
			start_block: 10,
			status: WorkerStatusType::Inactive,
			status_last_updated: 10,
			api: api_info.clone(),
			location: worker_location.clone(),
			specs: worker_specs.clone(),
			reputation: 0,
			last_status_check: current_timestamp,
		};

		// Dispatch a signed extrinsic.
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type,
			api_info.domain,
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Read pallet storage and assert an expected result.
		assert_eq!(
			pallet_edge_connect::WorkerClusters::<Test>::get((alice, 0)),
			Some(worker)
		);
	});
}

#[test]
fn it_fails_for_registering_duplicate_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		let domain_str = "127.0.0.1:3001";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let worker_type_0 = WorkerType::Docker;
		let worker_type_1 = WorkerType::Executable;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		let api_info = WorkerAPI { domain: domain };

		// Register the first worker
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_0.clone(),
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Try to register the same worker again
		assert_noop!(
			EdgeConnectModule::register_worker(
				RuntimeOrigin::signed(alice),
				worker_type_0,
				api_info.domain.clone(),
				latitude,
				longitude,
				ram,
				storage,
				cpu
			),
			Error::<Test>::WorkerExists
		);

		// Register the first worker
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_1.clone(),
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Try to register the same worker again
		assert_noop!(
			EdgeConnectModule::register_worker(
				RuntimeOrigin::signed(alice),
				worker_type_1,
				api_info.domain,
				latitude,
				longitude,
				ram,
				storage,
				cpu
			),
			Error::<Test>::WorkerExists
		);
	});
}

#[test]
fn it_works_for_removing_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		let domain_str = "127.0.0.1:3001";
		let domain_vec = domain_str.as_bytes().to_vec();
		let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		let worker_type_0 = WorkerType::Docker;
		let worker_type_1 = WorkerType::Executable;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		let api_info = WorkerAPI { domain: domain };

		// Register a worker first
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_0,
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		// Register a worker first
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_1,
			api_info.domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		// Remove the worker
		assert_ok!(EdgeConnectModule::remove_worker(
			RuntimeOrigin::signed(alice),
			WorkerType::Docker,
			0
		));
		// Remove the worker
		assert_ok!(EdgeConnectModule::remove_worker(
			RuntimeOrigin::signed(alice),
			WorkerType::Executable,
			1
		));

		// Assert that the worker no longer exists
		assert_eq!(
			pallet_edge_connect::WorkerClusters::<Test>::get((alice, 0)),
			None
		);
		// Assert that the worker no longer exists
		assert_eq!(
			pallet_edge_connect::ExecutableWorkers::<Test>::get((alice, 1)),
			None
		);
	});
}

#[test]
fn it_fails_for_removing_non_existent_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		// Attempt to remove a worker that doesn't exist
		assert_noop!(
			EdgeConnectModule::remove_worker(RuntimeOrigin::signed(alice), WorkerType::Docker, 0),
			Error::<Test>::WorkerDoesNotExist
		);

		// Attempt to remove a worker that doesn't exist
		assert_noop!(
			EdgeConnectModule::remove_worker(RuntimeOrigin::signed(alice), WorkerType::Executable, 0),
			Error::<Test>::WorkerDoesNotExist
		);
	});
}

#[test]
fn emiting_proper_event_for_registering_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let alice_first_worker_id = 0;
		let domain_str = "foobarkoo.com";
		let domain: BoundedVec<u8, ConstU32<128>> =
			BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap();
		let worker_type = WorkerType::Docker;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type,
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));
		System::assert_last_event(RuntimeEvent::EdgeConnectModule(Event::WorkerRegistered {
			creator: alice,
			worker: (alice, alice_first_worker_id),
			domain: domain,
		}));
	})
}

#[test]
fn it_works_for_changing_visibility() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let alice_first_worker_id = 0;
		let alice_second_worker_id = 1;
		let domain_str = "foobarkoo.com";
		let domain: BoundedVec<u8, ConstU32<128>> =
			BoundedVec::try_from(domain_str.as_bytes().to_vec()).unwrap();
		let worker_type_0 = WorkerType::Docker;
		let worker_type_1 = WorkerType::Executable;
		let latitude: Latitude = 590000;
		let longitude: Longitude = 120000;
		let ram: RamBytes = 100000000;
		let storage: StorageBytes = 100000000;
		let cpu: CpuCores = 12;

		System::set_block_number(10);
		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_0.clone(),
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		assert_ok!(EdgeConnectModule::register_worker(
			RuntimeOrigin::signed(alice),
			worker_type_1.clone(),
			domain.clone(),
			latitude,
			longitude,
			ram,
			storage,
			cpu
		));

		let _ = EdgeConnectModule::toggle_worker_visibility(
			RuntimeOrigin::signed(alice),
			worker_type_0.clone(),
			alice_first_worker_id,
			true,
		);

		let _ = EdgeConnectModule::toggle_worker_visibility(
			RuntimeOrigin::signed(alice),
			worker_type_1.clone(),
			alice_second_worker_id,
			true,
		);

		assert_eq!(
			pallet_edge_connect::WorkerClusters::<Test>::get((alice, alice_first_worker_id))
				.unwrap()
				.status,
			WorkerStatusType::Active
		);

		assert_eq!(
			pallet_edge_connect::ExecutableWorkers::<Test>::get((alice, alice_second_worker_id))
				.unwrap()
				.status,
			WorkerStatusType::Active
		);

		let _ = EdgeConnectModule::toggle_worker_visibility(
			RuntimeOrigin::signed(alice),
			worker_type_0,
			alice_first_worker_id,
			false,
		);

		let _ = EdgeConnectModule::toggle_worker_visibility(
			RuntimeOrigin::signed(alice),
			worker_type_1,
			alice_second_worker_id,
			false,
		);

		assert_eq!(
			pallet_edge_connect::WorkerClusters::<Test>::get((alice, alice_first_worker_id))
				.unwrap()
				.status,
			WorkerStatusType::Inactive
		);

		assert_eq!(
			pallet_edge_connect::ExecutableWorkers::<Test>::get((alice, alice_second_worker_id))
				.unwrap()
				.status,
			WorkerStatusType::Inactive
		);
	})
}

#[test]
fn it_fails_for_changing_visibility_on_nonexistant_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let alice_first_worker_id = 0;

		System::set_block_number(10);
		assert_noop!(
			EdgeConnectModule::toggle_worker_visibility(
				RuntimeOrigin::signed(alice),
				WorkerType::Docker,
				alice_first_worker_id,
				true
			),
			Error::<Test>::WorkerDoesNotExist
		);

		assert_noop!(
			EdgeConnectModule::toggle_worker_visibility(
				RuntimeOrigin::signed(alice),
				WorkerType::Executable,
				alice_first_worker_id,
				true
			),
			Error::<Test>::WorkerDoesNotExist
		);
	})
}

/*

	let domain_str = "some_api_domain.com";
	let domain_vec = domain_str.as_bytes().to_vec();
	let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();

	System::set_block_number(10);
	let alice = 0;
	let api_info = WorkerAPI { domain: domain };

	let worker = Worker {
		id: 0,
		owner: alice,
		start_block: 10,
		status: WorkerStatusType::Inactive,
		api: api_info.clone(),
	};

	// Dispatch a signed extrinsic.
	assert_ok!(EdgeConnectModule::register_worker(
		RuntimeOrigin::signed(alice),
		api_info.domain
	));
	// Read pallet storage and assert an expected result.
	assert_eq!(
		EdgeConnectModule::get_worker_clusters((alice, 0)),
		Some(worker)
	);

*/
