use crate::mock::*;
use crate::BalanceOf;
use frame_support::{assert_noop, assert_ok};

// Test for purchasing compute hours successfully
#[test]
fn it_works_for_purchasing_compute_hours() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Admin sets the price per hour to 100 units of balance
		let new_price: BalanceOf<Test> = 100u64.into();

		// Use the Sudo pallet to set the price per hour
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// Verify the price has been updated correctly in storage
		assert_eq!(pallet_payment::PricePerHour::<Test>::get(), new_price);

		// Admin uses Sudo to set the service provider account
		let service_provider_account = USER3;
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN),
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_service_provider_account {
					new_account: service_provider_account.clone()
				}
			)),
		));

		// Verify the service provider account is updated in storage
		assert_eq!(
			pallet_payment::ServiceProviderAccount::<Test>::get(),
			Some(service_provider_account)
		);

		// Store the initial balance of the user
		let initial_balance = Balances::free_balance(USER2);

		// Store the initial balance of the service provider account
		let service_provider_initial_balance = Balances::free_balance(USER3);
		println!(
			"service_provider_initial_balance {:?}",
			service_provider_initial_balance
		);

		// User2 purchases 10 compute hours
		let purchased_hours = 10u32;
		assert_ok!(PaymentModule::purchase_compute_hours(
			RuntimeOrigin::signed(USER2),
			purchased_hours
		));

		// Check that the compute hours have been correctly credited to the user
		assert_eq!(
			pallet_payment::ComputeHours::<Test>::get(USER2),
			purchased_hours
		);

		// Verify that the user's balance is reduced by the correct total cost
		let final_balance = Balances::free_balance(USER2);
		let total_cost = new_price.checked_mul(purchased_hours.into()).unwrap(); // 100 (price per hour) * 10 hours
		assert_eq!(final_balance, initial_balance - total_cost);

		// Store the initial balance of the service provider account
		let service_provider_final_balance = Balances::free_balance(USER3);
		assert_eq!(
			service_provider_final_balance,
			service_provider_initial_balance + total_cost
		);
		println!(
			"service_provider_final_balance {:?}",
			service_provider_final_balance
		);
	});
}

// Test to ensure purchasing zero hours fails
#[test]
fn it_fails_when_purchasing_zero_hours() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Admin sets the price per hour
		let new_price: BalanceOf<Test> = 100u64.into();

		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// User2 tries to purchase 0 hours and this should fail with InvalidHoursInput error
		assert_noop!(
			PaymentModule::purchase_compute_hours(RuntimeOrigin::signed(USER2), 0),
			crate::Error::<Test>::InvalidHoursInput
		);
	});
}

// Test to ensure purchasing hours fails if no service provider account is set
#[test]
fn it_fails_when_purchasing_hours_without_service_provider_account() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Admin sets the price per hour
		let new_price: BalanceOf<Test> = 100u64.into();

		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// User2 tries to purchase compute hours, but no service provider account is set
		let purchased_hours = 10u32;
		assert_noop!(
			PaymentModule::purchase_compute_hours(RuntimeOrigin::signed(USER2), purchased_hours),
			crate::Error::<Test>::ServiceProviderAccountNotFound
		);
	});
}

// Test to ensure purchasing with insufficient balance fails
#[test]
fn it_fails_when_purchasing_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Admin sets a high price per hour to trigger insufficient balance
		let new_price: BalanceOf<Test> = 10000u64.into();
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// User2 tries to purchase 10 hours, but balance is insufficient
		assert_noop!(
			PaymentModule::purchase_compute_hours(RuntimeOrigin::signed(USER2), 10),
			crate::Error::<Test>::InsufficientBalance
		);
	});
}

// Test for successful consumption of compute hours
#[test]
fn it_works_for_consuming_compute_hours() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Admin sets the price per hour
		let new_price: BalanceOf<Test> = 100u64.into();

		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// Verify the price has been updated correctly
		assert_eq!(pallet_payment::PricePerHour::<Test>::get(), new_price);

		// Admin uses Sudo to set the service provider account
		let service_provider_account = USER3;
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN),
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_service_provider_account {
					new_account: service_provider_account.clone()
				}
			)),
		));

		// Verify the service provider account is updated in storage
		assert_eq!(
			pallet_payment::ServiceProviderAccount::<Test>::get(),
			Some(service_provider_account)
		);

		// User2 purchases 10 compute hours
		let purchased_hours = 10u32;
		assert_ok!(PaymentModule::purchase_compute_hours(
			RuntimeOrigin::signed(USER2),
			purchased_hours
		));

		// Ensure compute hours are correctly credited
		assert_eq!(
			pallet_payment::ComputeHours::<Test>::get(USER2),
			purchased_hours
		);

		// User2 consumes 5 compute hours
		let consumed_hours = 5u32;
		assert_ok!(PaymentModule::consume_compute_hours(
			RuntimeOrigin::signed(USER2),
			consumed_hours
		));

		// Check the remaining compute hours after consumption
		let remaining_hours = purchased_hours - consumed_hours;
		assert_eq!(
			pallet_payment::ComputeHours::<Test>::get(USER2),
			remaining_hours
		);

		// Verify that the HoursConsumed event was emitted
		let expected_event =
			RuntimeEvent::PaymentModule(crate::Event::HoursConsumed(USER2, consumed_hours));
		assert!(System::events()
			.iter()
			.any(|record| { record.event == expected_event }));
	});
}

// Test to ensure consuming more hours than owned fails
#[test]
fn it_fails_when_consuming_more_hours_than_owned() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// Admin sets the price per hour
		let new_price: BalanceOf<Test> = 100u64.into();

		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// Verify the price has been updated in storage
		assert_eq!(pallet_payment::PricePerHour::<Test>::get(), new_price);

		// Admin uses Sudo to set the service provider account
		let service_provider_account = USER3;
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN),
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_service_provider_account {
					new_account: service_provider_account.clone()
				}
			)),
		));

		// Verify the service provider account is updated in storage
		assert_eq!(
			pallet_payment::ServiceProviderAccount::<Test>::get(),
			Some(service_provider_account)
		);

		// User2 purchases 10 compute hours
		let purchased_hours = 10u32;
		assert_ok!(PaymentModule::purchase_compute_hours(
			RuntimeOrigin::signed(USER2),
			purchased_hours
		));

		// Check that 10 hours were credited to the user
		assert_eq!(
			pallet_payment::ComputeHours::<Test>::get(USER2),
			purchased_hours
		);

		// User 2attempts to consume 20 hours, but only has 10
		assert_noop!(
			PaymentModule::consume_compute_hours(RuntimeOrigin::signed(USER2), 20),
			crate::Error::<Test>::InsufficientComputeHours
		);
	});
}

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

// Test to ensure that only the admin can set the price per hour
#[test]
fn admin_can_set_price_per_hour() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let new_price: BalanceOf<Test> = 100u64.into();

		// Admin uses Sudo to set the price per hour
		assert_ok!(Sudo::sudo(
			RuntimeOrigin::signed(ADMIN), // Sudo account ID
			Box::new(RuntimeCall::PaymentModule(
				crate::Call::set_price_per_hour { new_price }
			)),
		));

		// Check that the PricePerHourSet event was emitted
		let expected_event = RuntimeEvent::PaymentModule(crate::Event::PricePerHourSet(new_price));
		assert!(System::events()
			.iter()
			.any(|record| record.event == expected_event));

		// Verify the price is correctly updated in storage
		assert_eq!(pallet_payment::PricePerHour::<Test>::get(), new_price);
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

// Test to ensure non-admins cannot set the price per hour
#[test]
fn non_admin_cannot_set_price_per_hour() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let new_price: BalanceOf<Test> = 100u64.into();

		// Non-admin user (USER2) tries to set the price per hour, which should fail
		assert_noop!(
			PaymentModule::set_price_per_hour(RuntimeOrigin::signed(USER2), new_price),
			sp_runtime::DispatchError::BadOrigin
		);

		// Verify that the price has not been updated
		assert_eq!(pallet_payment::PricePerHour::<Test>::get(), 0);
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
