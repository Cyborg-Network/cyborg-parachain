#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	pallet_prelude::{ValueQuery, *},
	traits::Currency,
};
use frame_system::pallet_prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

//pub use cyborg_primitives::worker::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_system::Origin;

	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// Abstraction over the chain's currency system (e.g., balances or assets).
		type Currency: Currency<Self::AccountId>;

		// /// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn compute_hours)]
	pub type ComputeHours<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_per_hour)]
	pub type PricePerHour<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn service_provider_account)]
	pub type ServiceProviderAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		HoursPurchased(T::AccountId, u32, BalanceOf<T>),
		HoursConsumed(T::AccountId, u32),
		PricePerHourSet(BalanceOf<T>),
		ServiceProviderAccountSet(T::AccountId),
	}

	/// Pallet Errors
	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
		InsufficientComputeHours,
		ServiceProviderAccountNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_price_per_hour() )]
		pub fn set_price_per_hour(origin: OriginFor<T>, new_price: BalanceOf<T>) -> DispatchResult {
			ensure_root(origin)?;

			PricePerHour::<T>::put(new_price);

			Self::deposit_event(Event::PricePerHourSet(new_price));

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_service_provider_account() )]
		pub fn set_service_provider_account(
			origin: OriginFor<T>,
			new_account: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			ServiceProviderAccount::<T>::put(new_account.clone());

			Self::deposit_event(Event::ServiceProviderAccountSet(new_account));

			Ok(())
		}
	}
}
