#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_price_per_hour<T: Config>() -> Result<(), BenchmarkError> {
		// Initialize the new price to be set.
		let new_price: BalanceOf<T> = 100u32.into();

		// Start a benchmarking block. This block will be timed for performance.
		#[block]
		{
			// Loop to simulate multiple price setting operations for benchmarking.
			for i in 1u32..500 {
				// Set the price for compute hours in each iteration.
				let new_dummy_price = i;
				Pallet::<T>::set_price_per_hour(RawOrigin::Root.into(), new_dummy_price.into()).unwrap();
			}
			// Set the final price for compute hours.
			Pallet::<T>::set_price_per_hour(RawOrigin::Root.into(), new_price.clone()).unwrap();
		}

		// Verification code:
		// Ensure that the price was set correctly by asserting its value.
		assert_eq!(PricePerHour::<T>::get(), new_price);

		Ok(())
	}

	#[benchmark]
	fn set_service_provider_account<T: Config>() -> Result<(), BenchmarkError> {
		// Initialize a new account to set as the service provider.
		let new_account: T::AccountId = account("user_acount", 0, 0);

		// Start a benchmarking block for the operation.
		#[block]
		{
			// Loop to simulate multiple service provider account changes.
			for i in 1u32..500 {
				// Set the service provider account in each iteration.
				let new_dumny_account: T::AccountId = account("user_account_dummy", i, i);
				Pallet::<T>::set_service_provider_account(RawOrigin::Root.into(), new_dumny_account)
					.unwrap();
			}
			// Set the final service provider account.
			Pallet::<T>::set_service_provider_account(RawOrigin::Root.into(), new_account.clone())
				.unwrap();
		}

		// Verification code:
		// Ensure that the service provider account was set correctly.
		assert_eq!(ServiceProviderAccount::<T>::get(), Some(new_account));

		Ok(())
	}

	#[benchmark]
	fn purchase_compute_hours<T: Config>() -> Result<(), BenchmarkError> {
		// Set up the account balance
		// The runtime's ExistentialDeposit is configured to be 1_000_000_000;
		// We are ensuring the balance is set above the ExistentialDeposit.
		let balance: BalanceOf<T> = 1_00_000_000_000u128.saturated_into();

		// Set the price per hour for compute hours.
		let new_price: BalanceOf<T> = 10u32.saturated_into();
		Pallet::<T>::set_price_per_hour(RawOrigin::Root.into(), new_price).unwrap();
		assert_eq!(PricePerHour::<T>::get(), new_price);

		// Create a new service provider account and deposit the initial balance.
		let new_service_account: T::AccountId = account("service_account", 0, 0);
		let _ = T::Currency::make_free_balance_be(&new_service_account, balance);
		assert_eq!(T::Currency::free_balance(&new_service_account), balance);

		// Create a new user account and deposit the initial balance.
		let caller: T::AccountId = account::<T::AccountId>("caller", 0, 0);
		let _ = T::Currency::make_free_balance_be(&caller, balance);
		assert_eq!(T::Currency::free_balance(&caller), balance);

		// Set up initial service provider account
		Pallet::<T>::set_service_provider_account(RawOrigin::Root.into(), new_service_account.clone())
			.unwrap();
		assert_eq!(
			ServiceProviderAccount::<T>::get(),
			Some(new_service_account.clone())
		);

		// Start a benchmarking block for purchasing compute hours.
		#[block]
		{
			// Loop to simulate multiple compute hour purchase operations.
			for _i in 1u32..501 {
				Pallet::<T>::purchase_compute_hours(RawOrigin::Signed(caller.clone()).into(), 10u32)
					.unwrap();
			}
		}

		// Verification code:
		// Assert the final compute hours and balances after the purchases.
		assert_eq!(ComputeHours::<T>::get(&caller), 5_000u32);
		assert_eq!(
			T::Currency::free_balance(&new_service_account),
			100_000_050_000u128.saturated_into()
		);
		assert_eq!(
			T::Currency::free_balance(&caller),
			99_999_950_000u128.saturated_into()
		);

		Ok(())
	}

	#[benchmark]
	fn consume_compute_hours<T: Config>() -> Result<(), BenchmarkError> {
		// Set up the account balance
		// The runtime's ExistentialDeposit is configured to be 1_000_000_000;
		// We are ensuring the balance is set above the ExistentialDeposit.
		let balance: BalanceOf<T> = 1_00_000_000_000u128.saturated_into();

		// Set the price per hour for compute hours.
		let new_price: BalanceOf<T> = 10u32.into();
		Pallet::<T>::set_price_per_hour(RawOrigin::Root.into(), new_price).unwrap();
		assert_eq!(PricePerHour::<T>::get(), new_price);

		// Create a new service provider account and deposit the initial balance.
		let new_service_account: T::AccountId = account("service_account", 0, 0);
		let _ = T::Currency::deposit_creating(&new_service_account, balance);
		assert_eq!(T::Currency::free_balance(&new_service_account), balance);

		// Create a new user account and deposit the initial balance.
		let caller: T::AccountId = account("caller", 0, 0);
		let _ = T::Currency::deposit_creating(&caller, balance);
		assert_eq!(T::Currency::free_balance(&caller), balance);

		// Set up initial service provider account
		Pallet::<T>::set_service_provider_account(RawOrigin::Root.into(), new_service_account.clone())
			.unwrap();
		assert_eq!(
			ServiceProviderAccount::<T>::get(),
			Some(new_service_account.clone())
		);

		// Set up initial purchase computer hours
		Pallet::<T>::purchase_compute_hours(RawOrigin::Signed(caller.clone()).into(), 5_000u32)
			.unwrap();
		assert_eq!(ComputeHours::<T>::get(&caller), 5_000u32);

		// Start a benchmarking block for purchasing compute hours.
		#[block]
		{
			// Loop to simulate multiple compute hour purchase operations.
			for _i in 1u32..501 {
				Pallet::<T>::consume_compute_hours(RawOrigin::Signed(caller.clone()).into(), 10u32)
					.unwrap();
			}
		}

		// Verification code:
		// Assert the final compute hours and balances after the purchases.
		assert_eq!(ComputeHours::<T>::get(&caller), 0u32);
		assert_eq!(
			T::Currency::free_balance(&new_service_account),
			100_000_050_000u128.saturated_into()
		);
		assert_eq!(
			T::Currency::free_balance(&caller),
			99_999_950_000u128.saturated_into()
		);

		Ok(())
	}

	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
