#![cfg_attr(not(feature = "std"), no_std)]

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
use orml_traits::{OnNewData, CombineData};
use orml_oracle;
use frame_system;
// use crate::{Config, MomentOf, TimestampedValueOf};
use frame_support::{LOG_TARGET,traits::{Get, Time}};
// use sp_std::{marker, prelude::*};
use cyborg_primitives::oracle::{TimestampedValue,ProcessStatus};

pub type WorkerId = u64;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct StatusInstance<BlockNumber> {
    is_online: bool,
    is_available: bool,
    block: BlockNumber
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

        /// Maximum number of blocks or block range used to calculate average status 
        #[pallet::constant]
        type MaxBlockRangePeriod: Get<BlockNumberFor<Self>>;

        /// The percentage of active oracle entries needed to determine online status for worker
        #[pallet::constant]
        type ThresholdOnlineStatus: Get<u32>;

        #[pallet::constant]
        type MaxAggregateParamLength: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);


    #[pallet::storage]
    #[pallet::getter(fn last_updated_block)]
    pub type LastClearedBlock<T: Config> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    
	#[pallet::storage]
	#[pallet::getter(fn worker_status_entries)]
	pub type WorkerStatusEntriesPerPeriod<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		BoundedVec<StatusInstance<BlockNumberFor<T>>, T::MaxAggregateParamLength>,
		ValueQuery,
	>;

    #[pallet::storage]
    #[pallet::getter(fn submitted_per_period)]
    pub type SubmittedPerPeriod<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
	    (T::AccountId, WorkerId),
        OptionQuery,
    >;

    #[pallet::storage]
	#[pallet::getter(fn worker_status_result)]
	pub type ResultingWorkerStatus<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		ProcessStatus,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		WorkerRegistered {
			creator: T::AccountId,
		},
	}

	/// Pallet Errors
    pub enum Error {
        ExceedsMaxAggregateParamLength,
    }

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(now: BlockNumberFor<T>) {
            if LastClearedBlock::<T>::get() + T::MaxBlockRangePeriod::get() <= now {
				let clear_result_a = SubmittedPerPeriod::<T>::clear(500, None);
                let clear_result_b = WorkerStatusEntriesPerPeriod::<T>::clear(500, None);
                if clear_result_a.maybe_cursor == None && clear_result_b.maybe_cursor == None {
                    LastClearedBlock::<T>::set(now);
                }
                log::info!(
                    target: LOG_TARGET,
                        "Clearing map result for SubmittedPerPeriod: {:?}",
                        clear_result_a.deconstruct()
                );
                log::info!(
                    target: LOG_TARGET,
                        "Clearing map result for WorkerStatusEntriesPerPeriod: {:?}",
                        clear_result_b.deconstruct()
                );
			}
        }
    }

	impl<T: Config> Pallet<T> {
        fn derive_status_for_period() {
            for (key_worker, value_status_vec) in WorkerStatusMap::<T>::iter() {
                let mut total: u32 = 0;
                if 
            }
        }
	}

    impl<T: Config> OnNewData<T::AccountId, (T::AccountId, u64), ProcessStatus> for Pallet<T> {
        fn on_new_data(who: &T::AccountId, key: &(T::AccountId, u64), value: &ProcessStatus) {
            if SubmittedPerPeriod::<T>::get(who).is_some() {
                log::error!(
                    target: LOG_TARGET,
                        "A value for this period was already submitted by: {:?}",
                        who
                );
                return
            }
            WorkerStatusEntriesPerPeriod::<T>::mutate(&key, |status_vec| match status_vec.try_push(
                StatusInstance{
                is_online: value.online,
                is_available: value.available,
                block: <frame_system::Pallet<T>>::block_number()
            }) {
                Ok(()) => {}
                Err(_) => {
                    log::error!(
                    target: LOG_TARGET,
                        "Failed to push status instance value due to exceeded capacity. \
                        Value was submitted by: {:?}",
                        who
                    );
                }
            });
            SubmittedPerPeriod::<T>::set(who, Some(key.clone()));
        }
    }

    impl<T: Config + orml_oracle::Config> CombineData<(T::AccountId, WorkerId), TimestampedValue<T>>
        for Pallet<T>
    {
        fn combine_data(
            _key: &(T::AccountId, WorkerId),
            _values: Vec<TimestampedValue<T>>,
            _prev_value: Option<TimestampedValue<T>>,
        ) -> Option<TimestampedValue<T>> {
            None
        }
    }
}



