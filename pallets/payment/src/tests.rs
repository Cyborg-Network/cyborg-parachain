use crate::mock::*;
use crate::BalanceOf;
use frame_support::traits::fungible::Mutate;
use frame_support::{assert_noop, assert_ok};

// Test to ensure consuming zero hours fails
#[test]
fn it_fails_when_consuming_zero_hours() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// User2 attempts to consume 0 hours, which should fail
		let consumed_hours = 0u32;
		assert_noop!(
			PaymentModule::consume_compute_hours(RuntimeOrigin::signed(USER2), consumed_hours),
			crate::Error::<Test>::InvalidHoursInput
		);
	});
}

// Test to ensure the admin can set the service provider account
#[test]
fn admin_can_set_service_provider_account() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let new_account = USER3;

		// Admin uses Sudo to set the service provider account
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN),
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_service_provider_account {
					new_account: new_account.clone()
				}
			)),
		));

		// Verify the service provider account is updated in storage
		assert_eq!(
			pallet_payment::ServiceProviderAccount::<Test>::get(),
			Some(new_account)
		);

		// Check that the ServiceProviderAccountSet event was emitted
		let expected_event =
			RuntimeEvent::PaymentModule(crate::Event::ServiceProviderAccountSet(new_account.clone()));
		assert!(System::events()
			.iter()
			.any(|record| record.event == expected_event));
	});
}

// Test to ensure non-admins cannot set the service provider account
#[test]
fn non_admin_cannot_set_service_provider_account() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let new_account = USER3;

		// Non-admin user (USER2) tries to set the service provider account, which should fail
		assert_noop!(
			PaymentModule::set_service_provider_account(RuntimeOrigin::signed(USER2), new_account),
			sp_runtime::DispatchError::BadOrigin
		);

		// Verify that the service provider account has not been updated
		assert_eq!(pallet_payment::ServiceProviderAccount::<Test>::get(), None);
	});
}

#[test]
fn it_records_usage_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		pallet_edge_connect::AccountWorkers::<Test>::insert(USER2, 0);

		assert_ok!(PaymentModule::record_usage(
			RuntimeOrigin::signed(USER2),
			70,
			50,
			80
		));

		assert_eq!(
			pallet_payment::MinerUsage::<Test>::get(USER2),
			Some((70, 50, 80))
		);

		let expected_event =
			RuntimeEvent::PaymentModule(crate::Event::MinerUsageRecorded(USER2, 70, 50, 80));
		assert!(System::events().iter().any(|e| e.event == expected_event));
	});
}

#[test]
fn it_fails_when_usage_input_is_invalid() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Usage above 100% should fail
		pallet_edge_connect::AccountWorkers::<Test>::insert(USER2, 0);

		assert_noop!(
			PaymentModule::record_usage(RuntimeOrigin::signed(USER2), 120, 50, 80),
			crate::Error::<Test>::InvalidUsageInput
		);
	});
}

#[test]
fn it_distributes_rewards_to_miners() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Set the service provider account
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN),
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_service_provider_account { new_account: USER3 }
			)),
		));

		let reward: BalanceOf<Test> = 500u64.into();

		// Give some balance to USER3 (service provider)
		pallet_balances::Pallet::<Test>::set_balance(&USER3, 10_000u64.into());

		// Set pending rewards for two miners
		pallet_payment::MinerPendingRewards::<Test>::insert(USER4, reward);
		pallet_payment::MinerPendingRewards::<Test>::insert(USER2, reward);

		let initial_balance_user4 = Balances::free_balance(&USER4);
		let initial_balance_user2 = Balances::free_balance(&USER2);

		// Distribute rewards
		assert_ok!(PaymentModule::distribute_rewards(RuntimeOrigin::root()));

		// Check balances
		assert_eq!(
			Balances::free_balance(USER4),
			initial_balance_user4 + reward
		);
		assert_eq!(
			Balances::free_balance(USER2),
			initial_balance_user2 + reward
		);
		assert_eq!(Balances::free_balance(USER3), (10_000u64 - 2 * 500).into());

		// Verify events for both users
		let event1 = RuntimeEvent::PaymentModule(crate::Event::MinerRewarded(USER4, reward));
		let event2 = RuntimeEvent::PaymentModule(crate::Event::MinerRewarded(USER2, reward));
		assert!(System::events().iter().any(|e| e.event == event1));
		assert!(System::events().iter().any(|e| e.event == event2));
	});
}

#[test]
fn it_fails_to_distribute_if_provider_not_set() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			PaymentModule::distribute_rewards(RuntimeOrigin::root()),
			crate::Error::<Test>::ServiceProviderAccountNotFound
		);
	});
}

//Record usage overwrite behavior
#[test]
fn it_overwrites_existing_usage() {
	new_test_ext().execute_with(|| {
		pallet_edge_connect::AccountWorkers::<Test>::insert(USER2, 0);

		assert_ok!(PaymentModule::record_usage(
			RuntimeOrigin::signed(USER2),
			40,
			60,
			80
		));
		assert_eq!(
			pallet_payment::MinerUsage::<Test>::get(USER2),
			Some((40, 60, 80))
		);

		// Overwrite with new usage
		assert_ok!(PaymentModule::record_usage(
			RuntimeOrigin::signed(USER2),
			10,
			20,
			30
		));
		assert_eq!(
			pallet_payment::MinerUsage::<Test>::get(USER2),
			Some((10, 20, 30))
		);
	});
}

// No transfer if reward is zero during distribution
#[test]
fn it_skips_distribution_for_zero_rewards() {
	new_test_ext().execute_with(|| {
		// Set provider and give it balance
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN),
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_service_provider_account { new_account: USER3 }
			)),
		));
		pallet_balances::Pallet::<Test>::set_balance(&USER3, 10_000u64.into());

		// USER2 has 0 reward
		let zero: BalanceOf<Test> = 0u64.into();
		pallet_payment::MinerPendingRewards::<Test>::insert(USER2, zero);

		let balance_before = Balances::free_balance(USER2);

		assert_ok!(PaymentModule::distribute_rewards(RuntimeOrigin::root()));

		let balance_after = Balances::free_balance(USER2);
		assert_eq!(balance_after, balance_before);
	});
}

#[test]
fn it_fails_to_record_usage_if_not_a_worker() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// USER2 is NOT a registered worker
		assert_noop!(
			PaymentModule::record_usage(RuntimeOrigin::signed(USER2), 70, 50, 80),
			pallet_payment::Error::<Test>::NotRegisteredMiner
		);
	});
}

#[test]
fn subscribe_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let provider = 99;
		let fee_per_hour = 10;
		let hours = 5;

		pallet_payment::SubscriptionFee::<Test>::put(fee_per_hour);
		pallet_payment::ServiceProviderAccount::<Test>::put(provider);

		assert_ok!(PaymentModule::subscribe(RuntimeOrigin::signed(user), hours));
		assert_eq!(pallet_payment::ComputeHours::<Test>::get(user), hours);
	});
}

#[test]
fn subscribe_fails_if_already_subscribed() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let provider = 99;

		pallet_payment::SubscriptionFee::<Test>::put(10);
		pallet_payment::ServiceProviderAccount::<Test>::put(provider);
		pallet_payment::ComputeHours::<Test>::insert(user, 10);

		assert_noop!(
			PaymentModule::subscribe(RuntimeOrigin::signed(user), 5),
			pallet_payment::Error::<Test>::AlreadySubscribed
		);
	});
}

#[test]
fn add_hours_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let provider = 99;

		pallet_payment::SubscriptionFee::<Test>::put(10);
		pallet_payment::ServiceProviderAccount::<Test>::put(provider);
		pallet_payment::ComputeHours::<Test>::insert(user, 5);

		assert_ok!(PaymentModule::add_hours(RuntimeOrigin::signed(user), 5));
		assert_eq!(pallet_payment::ComputeHours::<Test>::get(user), 10);
	});
}

#[test]
fn add_hours_fails_if_not_subscribed() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let provider = 99;

		pallet_payment::SubscriptionFee::<Test>::put(10);
		pallet_payment::ServiceProviderAccount::<Test>::put(provider);

		assert_noop!(
			PaymentModule::add_hours(RuntimeOrigin::signed(user), 5),
			pallet_payment::Error::<Test>::SubscriptionExpired
		);
	});
}

#[test]
fn set_subscription_fee_works() {
	new_test_ext().execute_with(|| {
		let new_fee = 42;
		assert_ok!(PaymentModule::set_subscription_fee_per_hour(
			RuntimeOrigin::root(),
			new_fee
		));
		assert_eq!(pallet_payment::SubscriptionFee::<Test>::get(), new_fee);
	});
}

#[test]
fn set_subscription_fee_fails_with_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PaymentModule::set_subscription_fee_per_hour(RuntimeOrigin::root(), 0),
			pallet_payment::Error::<Test>::InvalidFee
		);
	});
}

#[test]
fn set_reward_rates_for_miner_works() {
	new_test_ext().execute_with(|| {
		let miner = USER2;
		let active = pallet_payment::RewardRates {
			cpu: 10,
			ram: 20,
			storage: 30,
		};
		let idle = pallet_payment::RewardRates {
			cpu: 1,
			ram: 1,
			storage: 1,
		};

		assert_ok!(PaymentModule::set_reward_rates_for_miner(
			RuntimeOrigin::root(),
			miner.clone(),
			active.clone(),
			idle.clone()
		));

		assert_eq!(
			pallet_payment::ActiveRewardRates::<Test>::get(&miner),
			Some(active)
		);
		assert_eq!(
			pallet_payment::IdleRewardRates::<Test>::get(&miner),
			Some(idle)
		);
	});
}

#[test]
fn reward_miner_new_works_with_active_and_idle_hours() {
	new_test_ext().execute_with(|| {
		let miner = USER3;

		// Setup reward rates
		let active = pallet_payment::RewardRates {
			cpu: 10,
			ram: 20,
			storage: 30,
		};
		let idle = pallet_payment::RewardRates {
			cpu: 1,
			ram: 1,
			storage: 1,
		};
		assert_ok!(PaymentModule::set_reward_rates_for_miner(
			RuntimeOrigin::root(),
			miner.clone(),
			active.clone(),
			idle.clone()
		));

		// Simulate miner usage
		pallet_payment::MinerUsage::<Test>::insert(&miner, (50, 50, 50)); // 50% CPU, RAM, Storage usage

		// Call reward_miner_new
		let active_hours = 2;
		let idle_hours = 3;
		assert_ok!(PaymentModule::reward_miner(
			RuntimeOrigin::root(),
			active_hours,
			idle_hours,
			miner.clone()
		));

		// Expected calculations
		let active_payout = active.cpu * 50 / 100 + active.ram * 50 / 100 + active.storage * 50 / 100;
		let active_reward = active_payout * active_hours as u128;

		let idle_payout = idle.cpu + idle.ram + idle.storage;
		let idle_reward = idle_payout * idle_hours as u128;

		let total_reward = active_reward + idle_reward;

		assert_eq!(
			pallet_payment::MinerPendingRewards::<Test>::get(&miner),
			total_reward
		);
	});
}

#[test]
fn reward_miner_new_fails_when_rates_not_set() {
	new_test_ext().execute_with(|| {
		let miner = USER4;

		// Insert some usage to skip usage check
		pallet_payment::MinerUsage::<Test>::insert(&miner, (40, 40, 40));

		// No rates inserted
		assert_noop!(
			PaymentModule::reward_miner(RuntimeOrigin::root(), 1, 1, miner.clone()),
			pallet_payment::Error::<Test>::RewardRateNotSet
		);
	});
}
