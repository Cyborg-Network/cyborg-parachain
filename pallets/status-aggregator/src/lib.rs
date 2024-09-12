#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// pub mod weights;
// pub use weights::*;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;
use orml_traits::OnNewData;
use frame_system;

pub type WorkerId = u64;

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct StatusInstance<BlockNumber> {
    is_online: bool,
    is_available: bool,
    block: BlockNumber
}

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct WorkerStatus {
    uptime: u8,
}

impl WorkerStatus {
    pub fn new(uptime: u8) -> Self {
        // Enforce maximum value constraint in the constructor
        let valid_uptime = if uptime > 100 { 100 } else { uptime };
        Self { uptime: valid_uptime }
    }

    pub fn set_uptime(&mut self, value: u8) {
        // Ensure the value does not exceed 100
        self.uptime = if value > 100 { 100 } else { value };
    }

    pub fn get_uptime(&self) -> u8 {
        self.uptime
    }
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

    use frame_system::WeightInfo; //remove later

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// /// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);


	/// Worker Cluster information
	#[pallet::storage]
	#[pallet::getter(fn worker_status)]
	pub type WorkerStatusMap<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		StatusInstance<BlockNumberFor<T>>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {

	}

	/// Pallet Errors
	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}


	impl<T: Config> Pallet<T> {

	}

    impl<T: Config> OnNewData<T::AccountId, (T::AccountId, u64), bool> for Pallet<T> {
        fn on_new_data(who: &T::AccountId, key: &(T::AccountId, u64), value: &bool) {

            WorkerStatusMap::<T>::set(key, Some(StatusInstance{
                is_online: *value,
                is_available: true,
                block: <frame_system::Pallet<T>>::block_number()
            }));
        }
    }
}
