use crate::{mock::*, Event};
use crate::{
	pallet::Config, LastClearedBlock, MinerStatusEntriesPerPeriod, ProcessStatusPercentages,
	ResultingMinerStatus, ResultingMinerStatusPercentages, StatusInstance, SubmittedPerPeriod,
};

use frame_support::{assert_ok, pallet_prelude::ConstU32, traits::OnFinalize, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;

use cyborg_primitives::{
	miner::*,
	oracle::{OracleMinerFormat, ProcessStatus},
};

#[test]
fn prevents_nonexistent_miner_storage() {
	new_test_ext().execute_with(|| {
		// initalize test variables
		let oracle_feeder_1: AccountId = 100;

		// miner for which status is to be updated
		let key_1: OracleMinerFormat = OracleMinerFormat {
			id: BoundedVec::try_from(vec![0u8]).unwrap(),
			miner_type: MinerType::Cloud,
		};

		// initial sanity check
		assert_eq!(
			SubmittedPerPeriod::<Test>::get((&oracle_feeder_1, &key_1)),
			false,
		);
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), vec![],);

		// 1. Verify value updated on new data submitted by oracle feeder
		let status_info = ProcessStatus {
			online: true,
			available: true,
		};

		StatusAggregator::on_new_data(&oracle_feeder_1, &key_1, &status_info);

		// verify no state changes
		let entries: BoundedVec<
			StatusInstance<BlockNumberFor<Test>>,
			<Test as Config>::MaxAggregateParamLength,
		> = BoundedVec::try_from(vec![]).unwrap();

		assert_eq!(
			SubmittedPerPeriod::<Test>::get((&oracle_feeder_1, &key_1)),
			false,
		);
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), entries);
	})
}
#[test]
fn on_new_data_works_as_expected() {
	new_test_ext().execute_with(|| {
		// initalize test variables
		let inital_block = 1;
		System::set_block_number(inital_block);

		let oracle_feeder_1: AccountId = 100;
		let oracle_feeder_2: AccountId = 200;
		let miner_addrs: Vec<AccountId> = [0, 1, 2].to_vec();
		// let miner_ids: Vec<Vec<u8>> = vec![
		// 	b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec(),
		// 	b"CL-22222222-dddd-eeee-ffff-0987654321cd".to_vec(),
		// 	b"CL-33333333-ffff-gggg-hhhh-abcdef123456".to_vec(),
		// ];

		// Corresponding BoundedVec<_, 64> for MinerId
		let bounded_miner_ids: Vec<MinerId> = vec![
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab"
				.to_vec()
				.try_into()
				.unwrap(),
			b"CL-22222222-dddd-eeee-ffff-0987654321cd"
				.to_vec()
				.try_into()
				.unwrap(),
			b"CL-33333333-ffff-gggg-hhhh-abcdef123456"
				.to_vec()
				.try_into()
				.unwrap(),
		];
		// basic miner spec
		let miner_type = MinerType::Cloud;
		let miner_latitude: Latitude = 590000;
		let miner_longitude: Longitude = 120000;
		let miner_ram: RamBytes = 100000000;
		let miner_storage: StorageBytes = 100000000;
		let miner_cpu: CpuCores = 12;

		// register miners
		for (miner, id_bytes) in miner_addrs.iter().zip(bounded_miner_ids.iter()) {
			let domain_str = "some_api_domain.".to_owned() + &id_bytes.len().to_string() + ".com"; // you can keep your original domain logic
			let domain_vec = domain_str.as_bytes().to_vec();
			let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();

			assert_ok!(EdgeConnectModule::register_miner(
				RuntimeOrigin::signed(*miner),
				miner_type.clone(),
				id_bytes.clone(),
				domain.clone(),
				miner_latitude,
				miner_longitude,
				miner_ram,
				miner_storage,
				miner_cpu
			));
		}

		// miner for which status is to be updated
		let key_1: OracleMinerFormat = OracleMinerFormat {
			id: bounded_miner_ids[0].clone(),
			miner_type: MinerType::Cloud,
		};

		// initial sanity check
		assert_eq!(
			SubmittedPerPeriod::<Test>::get((&oracle_feeder_1, &key_1)),
			false,
		);
		assert_eq!(
			SubmittedPerPeriod::<Test>::get((&oracle_feeder_2, &key_1)),
			false,
		);
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), vec![],);

		// 1. Verify value updated on new data submitted by oracle feeder
		let status_info = ProcessStatus {
			online: true,
			available: true,
		};
		StatusAggregator::on_new_data(&oracle_feeder_1, &key_1, &status_info);

		// verify state changes
		let resulting_status_instance_1 = StatusInstance {
			is_online: true,
			is_available: true,
			block: inital_block,
		};

		let mut entries: BoundedVec<
			StatusInstance<BlockNumberFor<Test>>,
			<Test as Config>::MaxAggregateParamLength,
		> = BoundedVec::try_from(vec![resulting_status_instance_1]).unwrap();

		assert_eq!(
			SubmittedPerPeriod::<Test>::get((&oracle_feeder_1, &key_1)),
			true,
		);
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), entries,);

		// 2. Prevents data submited to the same miner by the same oracle feeder for this given period
		StatusAggregator::on_new_data(
			&oracle_feeder_1,
			&key_1,
			&ProcessStatus {
				online: false,
				available: false,
			},
		);
		// check that state does not change
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), entries,);

		// simulate time change
		System::set_block_number(inital_block + 1);

		// 3. Allow a second oracle feeder entry to submit status for the same miner
		let resulting_status_instance_2 = StatusInstance {
			is_online: true,
			is_available: true,
			block: inital_block + 1,
		};
		entries
			.try_push(resulting_status_instance_2.clone())
			.unwrap();

		StatusAggregator::on_new_data(&oracle_feeder_2, &key_1, &status_info);

		assert_eq!(
			SubmittedPerPeriod::<Test>::get((&oracle_feeder_2, &key_1)),
			true,
		);
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), entries,);

		// 4. Fill up entry storage for a miner during a period to the limit, verify state is as expected
		let mut oracle_feeder_n: AccountId = 300;

		for _ in 2..10 {
			StatusAggregator::on_new_data(&oracle_feeder_n, &key_1, &status_info);
			entries
				.try_push(resulting_status_instance_2.clone())
				.unwrap();
			oracle_feeder_n += 1;
		}
		// check all entries filled
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), entries,);

		// fails on overflow
		StatusAggregator::on_new_data(&oracle_feeder_n, &key_1, &status_info);

		// verify no state change
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_1), entries,);

		// 5. Allow oracle feeders to submit for a new miners
		let key_2: OracleMinerFormat = OracleMinerFormat {
			id: bounded_miner_ids[2].clone(),
			miner_type: MinerType::Cloud,
		};
		let key_3: OracleMinerFormat = OracleMinerFormat {
			id: bounded_miner_ids[1].clone(),
			miner_type: MinerType::Cloud,
		};

		let status_info_2 = ProcessStatus {
			online: true,
			available: false,
		};
		let status_info_3 = ProcessStatus {
			online: false,
			available: false,
		};
		StatusAggregator::on_new_data(&oracle_feeder_1, &key_2, &status_info_2);
		StatusAggregator::on_new_data(&oracle_feeder_2, &key_3, &status_info_3);

		let resulting_status_instance_2 = StatusInstance {
			is_online: true,
			is_available: false,
			block: inital_block + 1,
		};

		let resulting_status_instance_3 = StatusInstance {
			is_online: false,
			is_available: false,
			block: inital_block + 1,
		};

		let entries_2: BoundedVec<
			StatusInstance<BlockNumberFor<Test>>,
			<Test as Config>::MaxAggregateParamLength,
		> = BoundedVec::try_from(vec![resulting_status_instance_2]).unwrap();
		let entries_3: BoundedVec<
			StatusInstance<BlockNumberFor<Test>>,
			<Test as Config>::MaxAggregateParamLength,
		> = BoundedVec::try_from(vec![resulting_status_instance_3]).unwrap();

		// assert correct state changes
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_2), entries_2,);
		assert_eq!(MinerStatusEntriesPerPeriod::<Test>::get(&key_3), entries_3,);
	});
}

#[test]
fn on_finalize_works_as_expected_for_docker_miners() {
	new_test_ext().execute_with(|| {
		// initial sanity check
		assert_eq!(LastClearedBlock::<Test>::get(), 0,);
		assert_eq!(
			SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len(),
			0,
		);
		assert_eq!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len(),
			0,
		);

		// initalize test variables
		let inital_block = 1;
		System::set_block_number(inital_block);

		StatusAggregator::on_finalize(inital_block);

		let oracle_feeder_1: AccountId = 100;
		let oracle_feeder_2: AccountId = 200;
		let miner_addrs: Vec<AccountId> = [0, 1, 2].to_vec();
		// let miner_ids: Vec<Vec<u8>> = vec![
		// 	b"CL-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec(),
		// 	b"CL-22222222-dddd-eeee-ffff-0987654321cd".to_vec(),
		// 	b"CL-33333333-ffff-gggg-hhhh-abcdef123456".to_vec(),
		// ];

		// Corresponding BoundedVec<_, 64> for MinerId
		let bounded_miner_ids: Vec<MinerId> = vec![
			b"CL-11111111-aaaa-bbbb-cccc-1234567890ab"
				.to_vec()
				.try_into()
				.unwrap(),
			b"CL-22222222-dddd-eeee-ffff-0987654321cd"
				.to_vec()
				.try_into()
				.unwrap(),
			b"CL-33333333-ffff-gggg-hhhh-abcdef123456"
				.to_vec()
				.try_into()
				.unwrap(),
		];

		// basic miner spec
		let miner_latitude: Latitude = 590000;
		let miner_longitude: Longitude = 120000;
		let miner_ram: RamBytes = 100000000;
		let miner_storage: StorageBytes = 100000000;
		let miner_cpu: CpuCores = 12;
		let miner_type = MinerType::Cloud;

		// register miners
		for (miner, id_bytes) in miner_addrs.iter().zip(bounded_miner_ids.iter()) {
			let domain_str = "some_api_domain.".to_owned() + &id_bytes.len().to_string() + ".com"; // you can keep your original domain logic
			let domain_vec = domain_str.as_bytes().to_vec();
			let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();

			assert_ok!(EdgeConnectModule::register_miner(
				RuntimeOrigin::signed(*miner),
				miner_type.clone(),
				id_bytes.clone(),
				domain.clone(),
				miner_latitude,
				miner_longitude,
				miner_ram,
				miner_storage,
				miner_cpu
			));
		}

		// miner for which status is to be updated
		let key_1: OracleMinerFormat = OracleMinerFormat {
			id: bounded_miner_ids[0].clone(),
			miner_type: MinerType::Cloud,
		};
		let key_2: OracleMinerFormat = OracleMinerFormat {
			id: bounded_miner_ids[1].clone(), 
			miner_type: MinerType::Cloud,
		};

		assert_ok!(EdgeConnectModule::update_operational_status(
			RuntimeOrigin::signed(miner_addrs[1]),
			MinerType::Cloud,
			bounded_miner_ids[0].clone(),
			OperationalStatus::Busy
		));

		// pallet edge connect inital storage sanity check
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Busy
		);
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.status_last_updated,
			inital_block
		);
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.status_last_updated,
			inital_block
		);

		// 1. Verify value updated on new data submitted by oracle feeder
		StatusAggregator::on_new_data(
			&oracle_feeder_1,
			&key_1,
			&ProcessStatus {
				online: true,
				available: true,
			},
		);
		StatusAggregator::on_new_data(
			&oracle_feeder_2,
			&key_1,
			&ProcessStatus {
				online: true,
				available: true,
			},
		);
		StatusAggregator::on_new_data(
			&oracle_feeder_1,
			&key_2,
			&ProcessStatus {
				online: true,
				available: false,
			},
		);
		StatusAggregator::on_new_data(
			&oracle_feeder_2,
			&key_2,
			&ProcessStatus {
				online: true,
				available: true,
			},
		);

		// ensure storage updates
		assert!(SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len() > 0);
		assert!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len()
				> 0
		);
		// check total entries submitted this period
		assert_eq!(
			SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len(),
			4,
		);
		// check miners with entries submitted this period
		assert_eq!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len(),
			2,
		);

		// 2. Ensure percentage values do not update until after MaxBlockRangePeriod
		// sanity check, no values updated after entries entered before next MaxBlockRangePeriod
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_1),
			ProcessStatusPercentages {
				online: 0,
				available: 0,
				last_block_processed: 0,
			},
		);
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_2),
			ProcessStatusPercentages {
				online: 0,
				available: 0,
				last_block_processed: 0,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_1),
			ProcessStatus {
				online: false,
				available: false,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_2),
			ProcessStatus {
				online: false,
				available: false,
			},
		);

		// 3. Ensure storage resets after each period
		// increase block time past MaxBlockRangePeriod
		System::set_block_number(MaxBlockRangePeriod::get() as u64);
		StatusAggregator::on_finalize(MaxBlockRangePeriod::get() as u64);

		System::assert_last_event(RuntimeEvent::StatusAggregator(Event::LastBlockUpdated {
			block_number: 5,
		}));

		// Validate update last cleared block
		assert_eq!(
			LastClearedBlock::<Test>::get(),
			MaxBlockRangePeriod::get() as u64,
		);
		// ensure mappings cleared
		assert_eq!(
			SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len(),
			0,
		);
		assert_eq!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len(),
			0,
		);

		// 4. Ensure correct calculation for percentage uptime and current miner status updates after MaxBlockRangePeriod
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_1),
			ProcessStatusPercentages {
				online: 100,
				available: 100,
				last_block_processed: 5,
			},
		);
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_2),
			ProcessStatusPercentages {
				online: 100,
				available: 50,
				last_block_processed: 5,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_1),
			ProcessStatus {
				online: true,
				available: true,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_2),
			ProcessStatus {
				online: true,
				available: false,
			},
		);

		System::assert_has_event(RuntimeEvent::StatusAggregator(
			Event::UpdateFromAggregatedMinerInfo {
				miner: key_1.id.clone(),
				online: true,
				available: true,
				last_block_processed: 5,
			},
		));

		System::assert_has_event(RuntimeEvent::StatusAggregator(
			Event::UpdateFromAggregatedMinerInfo {
				miner: key_2.id.clone(),
				online: true,
				available: false,
				last_block_processed: 5,
			},
		));

		// 5. Ensure pallet edge connect properly updates
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Busy
		);
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Available
		);

		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.status_last_updated,
			5
		);
		assert_eq!(
			pallet_edge_connect::CloudMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.status_last_updated,
			5
		);
	});
}

#[test]
fn on_finalize_works_as_expected_for_executable_miners() {
	new_test_ext().execute_with(|| {
		// initial sanity check
		assert_eq!(LastClearedBlock::<Test>::get(), 0,);
		assert_eq!(
			SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len(),
			0,
		);
		assert_eq!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len(),
			0,
		);

		// initalize test variables
		let inital_block = 1;
		System::set_block_number(inital_block);

		StatusAggregator::on_finalize(inital_block);

		let oracle_feeder_1: AccountId = 100;
		let oracle_feeder_2: AccountId = 200;
		let miner_addrs: Vec<AccountId> = [0, 1, 2].to_vec();
		// let miner_ids: Vec<Vec<u8>> = vec![
		// 	b"ED-11111111-aaaa-bbbb-cccc-1234567890ab".to_vec(),
		// 	b"ED-22222222-dddd-eeee-ffff-0987654321cd".to_vec(),
		// 	b"ED-33333333-ffff-gggg-hhhh-abcdef123456".to_vec(),
		// ];

		// Corresponding BoundedVec<_, 64> for MinerId
		let bounded_miner_ids: Vec<MinerId> = vec![
			b"ED-11111111-aaaa-bbbb-cccc-1234567890ab"
				.to_vec()
				.try_into()
				.unwrap(),
			b"ED-22222222-dddd-eeee-ffff-0987654321cd"
				.to_vec()
				.try_into()
				.unwrap(),
			b"ED-33333333-ffff-gggg-hhhh-abcdef123456"
				.to_vec()
				.try_into()
				.unwrap(),
		];

		// basic miner spec
		let miner_latitude: Latitude = 590000;
		let miner_longitude: Longitude = 120000;
		let miner_ram: RamBytes = 100000000;
		let miner_storage: StorageBytes = 100000000;
		let miner_cpu: CpuCores = 12;
		let miner_type = MinerType::Edge;

		// register miners
		// register miners — one-to-one mapping
		for (miner, id_bytes) in miner_addrs.iter().zip(bounded_miner_ids.iter()) {
			let domain_str = "some_api_domain.".to_owned() + &id_bytes.len().to_string() + ".com"; // you can keep your original domain logic
			let domain_vec = domain_str.as_bytes().to_vec();
			let domain: BoundedVec<u8, ConstU32<128>> = BoundedVec::try_from(domain_vec).unwrap();

			assert_ok!(EdgeConnectModule::register_miner(
				RuntimeOrigin::signed(*miner),
				miner_type.clone(),
				id_bytes.clone(),
				domain.clone(),
				miner_latitude,
				miner_longitude,
				miner_ram,
				miner_storage,
				miner_cpu
			));
		}

		// miner for which status is to be updated
		let key_1: OracleMinerFormat= OracleMinerFormat {
			id: bounded_miner_ids[0].clone(),
			miner_type: MinerType::Edge,
		};
		let key_2: OracleMinerFormat = OracleMinerFormat {
			id: bounded_miner_ids[1].clone(),
			miner_type: MinerType::Edge,
		};

		assert_ok!(EdgeConnectModule::update_operational_status(
			RuntimeOrigin::signed(miner_addrs[1]), 
			MinerType::Edge,
			bounded_miner_ids[1].clone(),
			OperationalStatus::Busy
		));

		// pallet edge connect inital storage sanity check
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Busy
		);
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.status_last_updated,
			inital_block
		);
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.status_last_updated,
			inital_block
		);

		// 1. Verify value updated on new data submitted by oracle feeder
		StatusAggregator::on_new_data(
			&oracle_feeder_1,
			&key_1,
			&ProcessStatus {
				online: true,
				available: true,
			},
		);
		StatusAggregator::on_new_data(
			&oracle_feeder_2,
			&key_1,
			&ProcessStatus {
				online: true,
				available: true,
			},
		);
		StatusAggregator::on_new_data(
			&oracle_feeder_1,
			&key_2,
			&ProcessStatus {
				online: true,
				available: false,
			},
		);
		StatusAggregator::on_new_data(
			&oracle_feeder_2,
			&key_2,
			&ProcessStatus {
				online: true,
				available: true,
			},
		);

		// ensure storage updates
		assert!(SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len() > 0);
		assert!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len()
				> 0
		);
		// check total entries submitted this period
		assert_eq!(
			SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len(),
			4,
		);
		// check miners with entries submitted this period
		assert_eq!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len(),
			2,
		);

		// 2. Ensure percentage values do not update until after MaxBlockRangePeriod
		// sanity check, no values updated after entries entered before next MaxBlockRangePeriod
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_1),
			ProcessStatusPercentages {
				online: 0,
				available: 0,
				last_block_processed: 0,
			},
		);
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_2),
			ProcessStatusPercentages {
				online: 0,
				available: 0,
				last_block_processed: 0,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_1),
			ProcessStatus {
				online: false,
				available: false,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_2),
			ProcessStatus {
				online: false,
				available: false,
			},
		);

		// 3. Ensure storage resets after each period
		// increase block time past MaxBlockRangePeriod
		System::set_block_number(MaxBlockRangePeriod::get() as u64);
		StatusAggregator::on_finalize(MaxBlockRangePeriod::get() as u64);

		System::assert_last_event(RuntimeEvent::StatusAggregator(Event::LastBlockUpdated {
			block_number: 5,
		}));

		// Validate update last cleared block
		assert_eq!(
			LastClearedBlock::<Test>::get(),
			MaxBlockRangePeriod::get() as u64,
		);
		// ensure mappings cleared
		assert_eq!(
			SubmittedPerPeriod::<Test>::iter().collect::<Vec<_>>().len(),
			0,
		);
		assert_eq!(
			MinerStatusEntriesPerPeriod::<Test>::iter()
				.collect::<Vec<_>>()
				.len(),
			0,
		);

		// 4. Ensure correct calculation for percentage uptime and current miner status updates after MaxBlockRangePeriod
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_1),
			ProcessStatusPercentages {
				online: 100,
				available: 100,
				last_block_processed: 5,
			},
		);
		assert_eq!(
			ResultingMinerStatusPercentages::<Test>::get(&key_2),
			ProcessStatusPercentages {
				online: 100,
				available: 50,
				last_block_processed: 5,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_1),
			ProcessStatus {
				online: true,
				available: true,
			},
		);
		assert_eq!(
			ResultingMinerStatus::<Test>::get(&key_2),
			ProcessStatus {
				online: true,
				available: false,
			},
		);

		System::assert_has_event(RuntimeEvent::StatusAggregator(
			Event::UpdateFromAggregatedMinerInfo {
				miner: key_1.id.clone(),
				online: true,
				available: true,
				last_block_processed: 5,
			},
		));

		System::assert_has_event(RuntimeEvent::StatusAggregator(
			Event::UpdateFromAggregatedMinerInfo {
				miner: key_2.id.clone(),
				online: true,
				available: false,
				last_block_processed: 5,
			},
		));

		// 5. Ensure pallet edge connect properly updates
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Available
		);
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.operational_status,
			OperationalStatus::Busy
		);
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_1.id.clone())
				.unwrap()
				.status_last_updated,
			5
		);
		assert_eq!(
			pallet_edge_connect::EdgeMiners::<Test>::get(key_2.id.clone())
				.unwrap()
				.status_last_updated,
			5
		);
	});
}
