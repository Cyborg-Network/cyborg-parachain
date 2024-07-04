
use crate::{mock::*, types::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_registering_domain() {
	new_test_ext().execute_with(|| {
		
		System::set_block_number(10);
		let alice = 0;
		let api_info = WorkerAPI {
			ip: None,
			domain: Some(100)
		};

		let worker = Worker {
			id: 0,
			owner: alice,
			start_block: 10,
			status: WorkerStatusType::Inactive,
			api: api_info.clone()
		};
		
		// Dispatch a signed extrinsic.
		assert_ok!(WorkerClustersModule::register_worker(RuntimeOrigin::signed(alice), api_info.ip, api_info.domain));
		// Read pallet storage and assert an expected result.
		assert_eq!(WorkerClustersModule::get_worker_clusters((alice,0)), Some(worker));
	});
}
