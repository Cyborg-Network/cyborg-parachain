use crate::mock::*;
use frame_support::BoundedVec;
use frame_support::{assert_noop, assert_ok};
use sp_std::convert::TryFrom;

const ADMIN: u64 = 2;
const USER: u64 = 1;

#[test]
fn it_works_for_purchasing_compute_hours() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let initial_balance = Balances::free_balance(USER);

		// Set price per hour (admin operation)
		assert_ok!(PaymentModule::set_price_per_hour(
			RuntimeOrigin::signed(2),
			100
		));

		/*




		// User 1 purchases 10 compute hours
		assert_ok!(PaymentModule::purchase_compute_hours(
			RuntimeOrigin::signed(USER),
			10
		));

		// Check if user's balance is reduced by the expected amount
		let final_balance = Balances::free_balance(USER);
		assert_eq!(final_balance, initial_balance - 1000); // 100 (price per hour) * 10 hours

		// Check if the compute hours are credited correctly
		assert_eq!(PaymentModule::compute_hours(USER), 10);

		// Verify event was emitted
		let expected_event = Event::PaymentModule(crate::Event::HoursPurchased(USER, 10, 1000));
		assert!(System::events().iter().any(|a| a.event == expected_event));

		*/
	});
}
/*
#[test]
fn it_fails_when_purchasing_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// Set price per hour (admin operation)
		assert_ok!(PaymentModule::set_price_per_hour(
			RuntimeOrigin::signed(2),
			1000
		)); // Price set very high to induce failure

		// User tries to purchase more hours than their balance allows
		assert_noop!(
			PaymentModule::purchase_compute_hours(RuntimeOrigin::signed(USER), 10),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn it_works_for_consuming_compute_hours() {
	new_test_ext().execute_with(|| {
		// Set price per hour (admin operation)
		assert_ok!(PaymentModule::set_price_per_hour(
			RuntimeOrigin::signed(2),
			100
		));

		// User 1 purchases 10 compute hours
		assert_ok!(PaymentModule::purchase_compute_hours(
			RuntimeOrigin::signed(USER),
			10
		));

		// User consumes 5 compute hours
		assert_ok!(PaymentModule::consume_compute_hours(
			RuntimeOrigin::signed(USER),
			5
		));

		// Check remaining compute hours
		assert_eq!(PaymentModule::compute_hours(USER), 5);

		// Verify event was emitted
		let expected_event = Event::PaymentModule(crate::Event::HoursConsumed(USER, 5));
		assert!(System::events().iter().any(|a| a.event == expected_event));
	});
}

#[test]
fn it_fails_when_consuming_more_hours_than_owned() {
	new_test_ext().execute_with(|| {
		// Set price per hour (admin operation)
		assert_ok!(PaymentModule::set_price_per_hour(
			RuntimeOrigin::signed(2),
			100
		));

		// User 1 purchases 5 compute hours
		assert_ok!(PaymentModule::purchase_compute_hours(
			RuntimeOrigin::signed(USER),
			5
		));

		// User tries to consume more hours than they have
		assert_noop!(
			PaymentModule::consume_compute_hours(RuntimeOrigin::signed(USER), 10),
			Error::<Test>::InsufficientComputeHours
		);
	});
}

#[test]
fn admin_can_set_price_per_hour() {
	new_test_ext().execute_with(|| {
		// Admin sets the price per hour
		assert_ok!(PaymentModule::set_price_per_hour(
			RuntimeOrigin::signed(ADMIN),
			200
		));

		// Verify the price has been updated in storage
		assert_eq!(PaymentModule::price_per_hour(), 200);
	});
}

#[test]
fn admin_can_set_service_provider_account() {
	new_test_ext().execute_with(|| {
		let new_service_provider = 3;

		// Admin sets the service provider account
		assert_ok!(PaymentModule::set_service_provider_account(
			RuntimeOrigin::signed(ADMIN),
			new_service_provider
		));

		// Verify the service provider account has been updated
		assert_eq!(
			PaymentModule::service_provider_account(),
			new_service_provider
		);
	});
}

#[test]
fn non_admin_cannot_set_price_per_hour() {
	new_test_ext().execute_with(|| {
		// Non-admin user tries to set the price per hour
		assert_noop!(
			PaymentModule::set_price_per_hour(RuntimeOrigin::signed(USER), 200),
			Error::<Test>::NotAuthorized
		);
	});
}

#[test]
fn non_admin_cannot_set_service_provider_account() {
	new_test_ext().execute_with(|| {
		let new_service_provider = 3;

		// Non-admin user tries to set the service provider account
		assert_noop!(
			PaymentModule::set_service_provider_account(
				RuntimeOrigin::signed(USER),
				new_service_provider
			),
			Error::<Test>::NotAuthorized
		);
	});
}

	*/
