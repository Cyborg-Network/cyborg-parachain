#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{traits::CheckedMul, ArithmeticError},
		traits::{Currency, ExistenceRequirement},
	};

	use super::*;

	/// Type alias to simplify balance-related operations.
	/// This maps the `Balance` type based on the associated `Currency` in the runtime config.
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Abstraction over the chain's currency system, allowing this pallet to interact with balances.
		type Currency: Currency<Self::AccountId>;

		/// A type representing the weights required by the dispatchable functions of this pallet.
		type WeightInfo: WeightInfo;
	}

	// Storage map that tracks the number of compute hours owned by each account.
	#[pallet::storage]
	pub type ComputeHours<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	// Storage value that holds the price per compute hour, defined by the admin.
	#[pallet::storage]
	pub type PricePerHour<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	// Storage that holds the service provider's account ID.
	#[pallet::storage]
	pub type ServiceProviderAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event triggered when compute hours are consumed.
		HoursConsumed(T::AccountId, u32),
		/// Event triggered when compute hours are purchased.
		HoursPurchased(T::AccountId, u32, BalanceOf<T>),
		/// Event triggered when the admin sets a new price per hour.
		PricePerHourSet(BalanceOf<T>),
		/// Event triggered when the admin sets a new service provider account.
		ServiceProviderAccountSet(T::AccountId),
	}

	/// Pallet Errors
	#[pallet::error]
	pub enum Error<T> {
		/// The user's balance is insufficient for the transaction.
		InsufficientBalance,
		/// The user does not have enough compute hours to consume.
		InsufficientComputeHours,
		/// The user has provided an invalid input for the number of hours (e.g., 0).
		InvalidHoursInput,
		/// The service provider account is not found.
		ServiceProviderAccountNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Allows the admin (root) to set the price per compute hour.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_price_per_hour() )]
		pub fn set_price_per_hour(origin: OriginFor<T>, new_price: BalanceOf<T>) -> DispatchResult {
			// Ensure the caller is root (admin).
			ensure_root(origin)?;

			// Update the price per hour in storage.
			PricePerHour::<T>::put(new_price);

			// Emit the event that the price has been set.
			Self::deposit_event(Event::PricePerHourSet(new_price));

			Ok(())
		}

		/// Allows the admin (root) to set the service provider's account.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_service_provider_account() )]
		pub fn set_service_provider_account(
			origin: OriginFor<T>,
			new_account: T::AccountId,
		) -> DispatchResult {
			// Ensure the caller is root (admin).
			ensure_root(origin)?;

			// Update the service provider account in storage.
			ServiceProviderAccount::<T>::put(new_account.clone());

			// Emit the event that the service provider account has been set.
			Self::deposit_event(Event::ServiceProviderAccountSet(new_account));

			Ok(())
		}

		/// Allows a user to purchase compute hours.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::purchase_compute_hours() )]
		pub fn purchase_compute_hours(origin: OriginFor<T>, hours: u32) -> DispatchResult {
			// Ensure the caller is a signed user.
			let who = ensure_signed(origin)?;

			// Ensure that the user isn't trying to purchase zero hours.
			ensure!(hours > 0, Error::<T>::InvalidHoursInput);

			// Retrieve the current price per hour from storage.
			let price_per_hour = PricePerHour::<T>::get();

			// Calculate the total cost for the requested compute hours.
			let total_cost = price_per_hour
				.checked_mul(&hours.into())
				.ok_or(ArithmeticError::Overflow)?;

			// Ensure the user has enough balance to cover the total cost.
			ensure!(
				T::Currency::free_balance(&who) >= total_cost,
				Error::<T>::InsufficientBalance
			);

			// Retrieve the service provider account
			let service_provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;

			// Deduct the amount from the user's balance and credit the service provider
			let _imbalance = T::Currency::transfer(
				&who,                            // From the user
				&service_provider,               // To the service provider
				total_cost,                      // The amount to transfer
				ExistenceRequirement::KeepAlive, // Ensure the accounts remain alive (existential amount enforced)
			)?;

			// Increase the user's compute hours in storage.
			ComputeHours::<T>::mutate(&who, |current_hours| {
				*current_hours += hours;
			});

			// Emit the event indicating the purchase of compute hours.
			Self::deposit_event(Event::HoursPurchased(who, hours, total_cost));

			Ok(())
		}

		/// Allows a user to consume compute hours.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::consume_compute_hours() )]
		pub fn consume_compute_hours(origin: OriginFor<T>, hours: u32) -> DispatchResult {
			// Ensure the caller is a signed user.
			let who = ensure_signed(origin)?;

			// Ensure that the user isn't trying to consume zero hours.
			ensure!(hours > 0, Error::<T>::InvalidHoursInput);

			// Retrieve the user's current compute hours.
			let current_hours = ComputeHours::<T>::get(&who);

			// Ensure the user has enough compute hours to consume.
			ensure!(current_hours >= hours, Error::<T>::InsufficientComputeHours);

			// Deduct the consumed hours from the user's total compute hours.
			ComputeHours::<T>::mutate(&who, |current| *current -= hours);

			// Emit the event indicating the consumption of compute hours.
			Self::deposit_event(Event::HoursConsumed(who, hours));

			Ok(())
		}
	}
}
