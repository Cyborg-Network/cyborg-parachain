#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_edge_connect::AccountWorkers;

pub mod weights;
pub use weights::*;
use sp_runtime::traits::Zero;
use sp_runtime::traits::CheckedAdd;
use log::info;
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
	pub trait Config: frame_system::Config + pallet_edge_connect::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Abstraction over the chain's currency system, allowing this pallet to interact with balances.
		type Currency: Currency<Self::AccountId>;

		/// A type representing the weights required by the dispatchable functions of this pallet.
		type WeightInfo: WeightInfo;
	}

	

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct RewardRates<Balance> {
		pub cpu: Balance,
		pub ram: Balance,
		pub storage: Balance,
	}

	#[pallet::storage]
    #[pallet::getter(fn subscription_fee)]
    pub(super) type SubscriptionFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;


	// Storage map that tracks the number of compute hours owned by each account.
	#[pallet::storage]
	pub type ComputeHours<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;


	// Storage that holds the service provider's account ID.
	#[pallet::storage]
	pub type ServiceProviderAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	/// Tracks the latest recorded usage percentages per miner (cpu%, ram%, storage%).
	#[pallet::storage]
	pub type MinerUsage<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (u8, u8, u8), OptionQuery>; // cpu, ram, storage usage percentages
	
	/// Accumulated pending rewards for each miner.
	#[pallet::storage]
	pub type MinerPendingRewards<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn active_reward_rates)]
	pub type ActiveRewardRates<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		RewardRates<BalanceOf<T>>, 
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn idle_reward_rates)]
	pub type IdleRewardRates<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		RewardRates<BalanceOf<T>>, 
		OptionQuery
	>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event triggered when compute hours are consumed.
		HoursConsumed(T::AccountId, u32),
		
		
		/// Event triggered when the admin sets a new service provider account.
		ServiceProviderAccountSet(T::AccountId),
		
		/// Event triggered when a miner's usage has been recorded.
		MinerUsageRecorded(T::AccountId, u8, u8, u8),
		/// Event triggered when a miner has been rewarded.
		MinerRewarded(T::AccountId, BalanceOf<T>),
		SubscriptionFeeSet(BalanceOf<T>),
		ConsumerSubscribed(T::AccountId, BalanceOf<T>, u32),
		SubscriptionRenewed(T::AccountId,u32),
		RewardRatesUpdated {
			miner: T::AccountId,
			active: RewardRates<BalanceOf<T>>,
			idle: RewardRates<BalanceOf<T>>,
		},
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
		
		/// Invalid usage percentages were provided (must be between 0 and 100).
		InvalidUsageInput,
		/// When a someone who is not a miner call miner permissioned extrinsics
		NotRegisteredMiner,

		InvalidFee,

		AlreadySubscribed,
        SubscriptionExpired,
		RewardRateNotSet,

	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		

		/// Allows the admin (root) to set the service provider's account.
		#[pallet::call_index(0)]
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

		
		/// Allows a user to consume compute hours.
		#[pallet::call_index(1)]
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

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn set_reward_rates_for_miner(
			origin: OriginFor<T>,
			miner: T::AccountId,
			active: RewardRates<BalanceOf<T>>,
			idle: RewardRates<BalanceOf<T>>
		) -> DispatchResult {
			ensure_root(origin)?;

			ActiveRewardRates::<T>::insert(&miner, active.clone());
			IdleRewardRates::<T>::insert(&miner, idle.clone());

			Self::deposit_event(Event::RewardRatesUpdated {
				miner,
				active,
				idle,
			});

			Ok(())
		}



		/// Record resource usage percentages (cpu, ram, storage) for a miner.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::record_usage() )]
		pub fn record_usage(origin: OriginFor<T>, cpu: u8, ram: u8, storage: u8) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				pallet_edge_connect::Pallet::<T>::account_workers(&who).is_some(),
				Error::<T>::NotRegisteredMiner
			);
			// ensure!(Self::is_registered_miner(&who), Error::<T>::NotRegisteredMiner);
			ensure!(cpu <= 100 && ram <= 100 && storage <= 100, Error::<T>::InvalidUsageInput);
			MinerUsage::<T>::insert(&who, (cpu, ram, storage));
			Self::deposit_event(Event::MinerUsageRecorded(who, cpu, ram, storage));
			Ok(())
		}
		
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::reward_miner())]
		pub fn reward_miner(
			origin: OriginFor<T>,
			active_hours: u32,
			idle_hours: u32,
			miner: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			let (cpu_usage, ram_usage, storage_usage) =
				MinerUsage::<T>::get(&miner).ok_or(Error::<T>::InvalidUsageInput)?;

			let active_rates = ActiveRewardRates::<T>::get(&miner).ok_or(Error::<T>::RewardRateNotSet)?;
			let idle_rates = IdleRewardRates::<T>::get(&miner).ok_or(Error::<T>::RewardRateNotSet)?;

			// Calculate active reward
			let active_payout = active_rates.cpu * cpu_usage.into() / 100u32.into()
				+ active_rates.ram * ram_usage.into() / 100u32.into()
				+ active_rates.storage * storage_usage.into() / 100u32.into();

			let active_reward = active_payout
				.checked_mul(&active_hours.into())
				.ok_or(ArithmeticError::Overflow)?;

			// Idle payout ignores usage percentages — paid at flat rate
			let idle_payout = idle_rates.cpu + idle_rates.ram + idle_rates.storage;

			let idle_reward = idle_payout
				.checked_mul(&idle_hours.into())
				.ok_or(ArithmeticError::Overflow)?;

			let total_reward = active_reward
				.checked_add(&idle_reward)
				.ok_or(ArithmeticError::Overflow)?;

			MinerPendingRewards::<T>::mutate(&miner, |pending| *pending += total_reward);

			Self::deposit_event(Event::MinerRewarded(miner, total_reward));
			Ok(())
		}



		/// Distribute all pending rewards to miners from the service provider's balance.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::distribute_rewards())]
		pub fn distribute_rewards(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			let provider = ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;
			for (miner, reward) in MinerPendingRewards::<T>::drain() {
				if reward.is_zero() {
					continue;
				}
				T::Currency::transfer(&provider, &miner, reward, ExistenceRequirement::KeepAlive)?;
				info!("{:?} rewarded with {:?} Native Coin",miner,reward);
				Self::deposit_event(Event::MinerRewarded(miner, reward));

			}
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn subscribe(origin: OriginFor<T>, hours: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(ComputeHours::<T>::get(&who) == 0 , Error::<T>::AlreadySubscribed);
			let fee_per_hour = SubscriptionFee::<T>::get();
			let total_fee = fee_per_hour.checked_mul(&hours.into()).ok_or(Error::<T>::InvalidFee)?;
			ensure!(T::Currency::free_balance(&who) >= total_fee, Error::<T>::InsufficientBalance);
			let provider = ServiceProviderAccount::<T>::get().ok_or(Error::<T>::SubscriptionExpired)?;
			T::Currency::transfer(&who, &provider, total_fee, ExistenceRequirement::KeepAlive)?;
			ComputeHours::<T>::insert(&who, hours);
			Self::deposit_event(Event::ConsumerSubscribed(who, total_fee, hours));
			Ok(())
		}
		
		#[pallet::call_index(7)]
		#[pallet::weight(10_000)]
		pub fn add_hours(origin: OriginFor<T>, extra_hours: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(ComputeHours::<T>::contains_key(&who), Error::<T>::SubscriptionExpired);
			let fee_per_hour = SubscriptionFee::<T>::get();
			let total_fee = fee_per_hour.checked_mul(&extra_hours.into()).ok_or(Error::<T>::InvalidFee)?;
			ensure!(T::Currency::free_balance(&who) >= total_fee, Error::<T>::InsufficientBalance);
			let provider = ServiceProviderAccount::<T>::get().ok_or(Error::<T>::SubscriptionExpired)?;
			T::Currency::transfer(&who, &provider, total_fee, ExistenceRequirement::KeepAlive)?;
			ComputeHours::<T>::mutate(&who, |hours| {
				*hours += extra_hours;
			});
			Self::deposit_event(Event::SubscriptionRenewed(who, extra_hours));
			Ok(())
		}
		
		#[pallet::call_index(8)]
		#[pallet::weight(10_000)]
		pub fn set_subscription_fee_per_hour(origin: OriginFor<T>, new_fee_per_hour: BalanceOf<T>) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(new_fee_per_hour > Zero::zero(), Error::<T>::InvalidFee);
			SubscriptionFee::<T>::put(new_fee_per_hour);
			Self::deposit_event(Event::SubscriptionFeeSet(new_fee_per_hour));
			Ok(())
		}
		


	}


}