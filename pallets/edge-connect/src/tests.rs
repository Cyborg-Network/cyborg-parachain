
use frame_support::BoundedVec;
use frame_support::sp_runtime::traits::ConstU32;
use crate::{mock::*, types::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryFrom;

#[test]
fn it_works_for_registering_domain() {
	new_test_ext().execute_with(|| {

		let domain_str = "https://some_api_domain.com";
        let domain_vec = domain_str.as_bytes().to_vec();
        let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();
		
		System::set_block_number(10);
		let alice = 0;
		let api_info = WorkerAPI {
			ip: None,
			domain: Some(domain),
		};

		let worker = Worker {
			id: 0,
			owner: alice,
			start_block: 10,
			status: WorkerStatusType::Inactive,
			api: api_info.clone()
		};
		
		// Dispatch a signed extrinsic.
		assert_ok!(edgeConnectModule::register_worker(RuntimeOrigin::signed(alice), api_info.ip, api_info.domain));
		// Read pallet storage and assert an expected result.
		assert_eq!(edgeConnectModule::get_worker_clusters((alice,0)), Some(worker));
	});
}

#[test]
fn it_works_for_registering_ip() {
	new_test_ext().execute_with(|| {
		System::set_block_number(10);
		let alice = 0;
		let api_info = WorkerAPI {
			ip: Some(Ip { ipv4: Some(125), ipv6: None, port: 123}),
			domain: None
		};

		let worker = Worker {
			id: 0,
			owner: alice,
			start_block: 10,
			status: WorkerStatusType::Inactive,
			api: api_info.clone()
		};
		
		// Dispatch a signed extrinsic.
		assert_ok!(edgeConnectModule::register_worker(RuntimeOrigin::signed(alice), api_info.ip, api_info.domain));
		// Read pallet storage and assert an expected result.
		assert_eq!(edgeConnectModule::get_worker_clusters((alice, 0)), Some(worker));
	});
}

#[test]
fn it_fails_for_registering_without_ip_or_domain() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		// Dispatch a signed extrinsic.
		assert_noop!(
			edgeConnectModule::register_worker(RuntimeOrigin::signed(alice), None, None),
			Error::<Test>::WorkerRegisterMissingIpOrDomain
		);
	});
}

#[test]
fn it_fails_for_registering_duplicate_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let api_info = WorkerAPI {
			ip: Some(Ip { ipv4: Some(125), ipv6: None, port: 123}),
			domain: None
		};

		// Register the first worker
		assert_ok!(edgeConnectModule::register_worker(RuntimeOrigin::signed(alice), api_info.ip.clone(), api_info.domain.clone()));
		// Try to register the same worker again
		assert_noop!(
			edgeConnectModule::register_worker(RuntimeOrigin::signed(alice), api_info.ip, api_info.domain),
			Error::<Test>::WorkerExists
		);
	});
}

#[test]
fn it_works_for_removing_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;
		let api_info = WorkerAPI {
			ip: Some(Ip { ipv4: Some(125), ipv6: None, port: 123}),
			domain: None
		};

		// Register a worker first
		assert_ok!(edgeConnectModule::register_worker(RuntimeOrigin::signed(alice), api_info.ip.clone(), api_info.domain.clone()));

		// Remove the worker
		assert_ok!(edgeConnectModule::remove_worker(RuntimeOrigin::signed(alice), 0));
		// Assert that the worker no longer exists
		assert_eq!(edgeConnectModule::get_worker_clusters((alice, 0)), None);
	});
}

#[test]
fn it_fails_for_removing_non_existent_worker() {
	new_test_ext().execute_with(|| {
		let alice = 0;

		// Attempt to remove a worker that doesn't exist
		assert_noop!(
			edgeConnectModule::remove_worker(RuntimeOrigin::signed(alice), 0),
			Error::<Test>::WorkerDoesNotExist
		);
	});
}