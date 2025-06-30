#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
use cyborg_primitives::payment::RewardRates;
use log::info;
use sp_runtime::traits::CheckedAdd;

use sp_runtime::traits::Zero;
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
	use sp_std::vec::Vec;

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

		// This will be added when we get the green signal

		// Default idle reward rate (per hour) for CPU usage when the miner is idle.
		// #[pallet::constant]
		// type IdleCpuRate: Get<BalanceOf<Self>>;

		// Default idle reward rate (per hour) for RAM usage when the miner is idle.
		// #[pallet::constant]
		// type IdleRamRate: Get<BalanceOf<Self>>;

		// Default idle reward rate (per hour) for Storage usage when the miner is idle.
		// #[pallet::constant]
		// type IdleStorageRate: Get<BalanceOf<Self>>;

		// The on-chain account ID of the Conductor server, responsible for fiat billing and orchestration.
		// #[pallet::constant]
		// type ConductorAccount: Get<Self::AccountId>;

		/// Maximum length for payment IDs
		#[pallet::constant]
		type MaxPaymentIdLength: Get<u32>;
	}

	/// Storage for mapping Stripe payment IDs to on-chain accounts
	#[pallet::storage]
	pub type StripePayments<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxPaymentIdLength>,
		T::AccountId,
		OptionQuery,
	>;

	/// Storage for pending FIAT payouts to miners
	#[pallet::storage]
	pub type MinerFiatPayouts<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Storage for conversion rate between compute hours and FIAT
	#[pallet::storage]
	pub type FiatConversionRate<T: Config> = StorageValue<_, (u64, BalanceOf<T>), ValueQuery>; // (fiat_cents, native_tokens)

	/// Storage for global per-hour subscription fee.
	#[pallet::storage]
	#[pallet::getter(fn subscription_fee)]
	pub(super) type SubscriptionFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Storage map that tracks the number of compute hours owned by each account.
	#[pallet::storage]
	pub type ComputeHours<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	/// Storage that holds the service provider's account ID.
	#[pallet::storage]
	pub type ServiceProviderAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	/// Store the latest recorded usage (cpu%, ram%, storage%) for miners.
	#[pallet::storage]
	pub type MinerUsage<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (u8, u8, u8), OptionQuery>; // cpu, ram, storage usage percentages

	/// Store rewards waiting to be distributed to miners.
	#[pallet::storage]
	pub type MinerPendingRewards<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Store custom reward rates when miner is active.
	#[pallet::storage]
	#[pallet::getter(fn active_reward_rates)]
	pub type ActiveRewardRates<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RewardRates<BalanceOf<T>>, OptionQuery>;

	/// Store custom reward rates when miner is idle.
	#[pallet::storage]
	#[pallet::getter(fn idle_reward_rates)]
	pub type IdleRewardRates<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RewardRates<BalanceOf<T>>, OptionQuery>;

	/// Event declarations for extrinsic calls.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		HoursConsumed(T::AccountId, u32), // Emitted when compute hours are used.
		ServiceProviderAccountSet(T::AccountId), // When admin sets provider.
		MinerUsageRecorded(T::AccountId, u8, u8, u8), // Usage data recorded.
		MinerRewarded(T::AccountId, BalanceOf<T>), // Reward given to a miner.
		SubscriptionFeeSet(BalanceOf<T>), // Admin set fee per hour.
		ConsumerSubscribed(T::AccountId, BalanceOf<T>, u32), // New subscription made.
		SubscriptionRenewed(T::AccountId, u32), // User adds hours.
		RewardRatesUpdated {
			miner: T::AccountId,
			active: RewardRates<BalanceOf<T>>,
			idle: RewardRates<BalanceOf<T>>,
		}, // When admin updates reward rates.
		FiatPaymentProcessed(T::AccountId, u32), // Account and compute hours added
		MinerFiatPayoutCreated(T::AccountId, BalanceOf<T>), // Miner payout record created
		FiatConversionRateUpdated(u64, BalanceOf<T>), // Rate updated (cents per native token)
		RemainingHoursQueried(T::AccountId, u32), 
	}

	/// Custom pallet errors.
	#[pallet::error]
	pub enum Error<T> {
		// Admin sets the global subscription cost per compute hour.W
		InsufficientBalance,            // User doesn't have enough tokens.
		InsufficientComputeHours,       // Not enough hours left to use.
		InvalidHoursInput,              // Invalid hours requested (e.g., 0).
		ServiceProviderAccountNotFound, // Service provider isn't set.
		InvalidUsageInput,              // Usage percentages out of bounds.
		NotRegisteredMiner,             // Miner not recognized by Edge Connect.
		InvalidFee,                     // Fee value is zero or invalid.
		AlreadySubscribed,              // User already subscribed.
		SubscriptionExpired,            // User has no subscription.
		RewardRateNotSet,               // Reward rate missing for a miner.
		InvalidStripePaymentId,
		FiatConversionRateNotSet,
	}

	/// Declare callable extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
			From<u64>,
	{
		/// Set the account that receives all payments.
		/// Can only be set by root user
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

		/// Admin sets a miner's reward rates for active and idle states.
		/// In future we idle rates will be static , will be set through the runtime configuration
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_reward_rates_for_miner())]
		pub fn set_reward_rates_for_miner(
			origin: OriginFor<T>,
			miner: T::AccountId,
			active: RewardRates<BalanceOf<T>>,
			idle: RewardRates<BalanceOf<T>>,
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

		/// Called by a registered miner to report their usage.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::record_usage() )]
		pub fn record_usage(origin: OriginFor<T>, cpu: u8, ram: u8, storage: u8) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				pallet_edge_connect::Pallet::<T>::account_workers(&who).is_some(),
				Error::<T>::NotRegisteredMiner
			);
			ensure!(
				cpu <= 100 && ram <= 100 && storage <= 100,
				Error::<T>::InvalidUsageInput
			);
			MinerUsage::<T>::insert(&who, (cpu, ram, storage));
			Self::deposit_event(Event::MinerUsageRecorded(who, cpu, ram, storage));
			Ok(())
		}

		/// Reward a miner for a given number of active and idle hours.
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

		/// Transfer all pending rewards to miners from the provider account.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::distribute_rewards())]
		pub fn distribute_rewards(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			let provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;
			for (miner, reward) in MinerPendingRewards::<T>::drain() {
				if reward.is_zero() {
					continue;
				}
				T::Currency::transfer(&provider, &miner, reward, ExistenceRequirement::KeepAlive)?;
				info!("{:?} rewarded with {:?} Native Coin", miner, reward);
				Self::deposit_event(Event::MinerRewarded(miner, reward));
			}
			Ok(())
		}

		/// Allows a new user to subscribe to compute by paying upfront.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::subscribe())]
		pub fn subscribe(origin: OriginFor<T>, hours: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				ComputeHours::<T>::get(&who) == 0,
				Error::<T>::AlreadySubscribed
			);
			let fee_per_hour = SubscriptionFee::<T>::get();
			let total_fee = fee_per_hour
				.checked_mul(&hours.into())
				.ok_or(Error::<T>::InvalidFee)?;
			ensure!(
				T::Currency::free_balance(&who) >= total_fee,
				Error::<T>::InsufficientBalance
			);
			let provider = ServiceProviderAccount::<T>::get().ok_or(Error::<T>::SubscriptionExpired)?;
			T::Currency::transfer(&who, &provider, total_fee, ExistenceRequirement::KeepAlive)?;
			ComputeHours::<T>::insert(&who, hours);
			Self::deposit_event(Event::ConsumerSubscribed(who, total_fee, hours));
			Ok(())
		}

		/// Lets an existing user add more hours to their subscription.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_hours())]
		pub fn add_hours(origin: OriginFor<T>, extra_hours: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				ComputeHours::<T>::contains_key(&who),
				Error::<T>::SubscriptionExpired
			);
			let fee_per_hour = SubscriptionFee::<T>::get();
			let total_fee = fee_per_hour
				.checked_mul(&extra_hours.into())
				.ok_or(Error::<T>::InvalidFee)?;
			ensure!(
				T::Currency::free_balance(&who) >= total_fee,
				Error::<T>::InsufficientBalance
			);
			let provider = ServiceProviderAccount::<T>::get().ok_or(Error::<T>::SubscriptionExpired)?;
			T::Currency::transfer(&who, &provider, total_fee, ExistenceRequirement::KeepAlive)?;
			ComputeHours::<T>::mutate(&who, |hours| {
				*hours += extra_hours;
			});
			Self::deposit_event(Event::SubscriptionRenewed(who, extra_hours));
			Ok(())
		}

		/// Admin sets the global subscription cost per compute hour.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_subscription_fee_per_hour())]
		pub fn set_subscription_fee_per_hour(
			origin: OriginFor<T>,
			new_fee_per_hour: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(new_fee_per_hour > Zero::zero(), Error::<T>::InvalidFee);
			SubscriptionFee::<T>::put(new_fee_per_hour);
			Self::deposit_event(Event::SubscriptionFeeSet(new_fee_per_hour));
			Ok(())
		}

		/// Admin sets the conversion rate between FIAT (cents) and native tokens
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_fiat_conversion_rate())]
		pub fn set_fiat_conversion_rate(
			origin: OriginFor<T>,
			fiat_cents: u64,
			native_tokens: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				fiat_cents > 0 && native_tokens > Zero::zero(),
				Error::<T>::InvalidFee
			);
			FiatConversionRate::<T>::put((fiat_cents, native_tokens));
			Self::deposit_event(Event::FiatConversionRateUpdated(fiat_cents, native_tokens));
			Ok(())
		}

		/// Process a FIAT payment and allocate compute hours
		#[pallet::call_index(10)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::process_fiat_payment())]
		pub fn process_fiat_payment(
			origin: OriginFor<T>,
			payment_id: Vec<u8>,
			account: T::AccountId,
			fiat_amount_cents: u64,
		) -> DispatchResult {
			ensure_root(origin)?;

			let bounded_payment_id: BoundedVec<u8, T::MaxPaymentIdLength> = payment_id
				.try_into()
				.map_err(|_| Error::<T>::InvalidStripePaymentId)?;

			ensure!(
				!StripePayments::<T>::contains_key(&bounded_payment_id),
				Error::<T>::InvalidStripePaymentId
			);

			ensure!(
				FiatConversionRate::<T>::exists(),
				Error::<T>::FiatConversionRateNotSet
			);
			let (cents_per_token, native_per_token) = FiatConversionRate::<T>::get();

			let compute_hours = fiat_amount_cents / 100;

			let native_value = native_per_token
				.checked_mul(&BalanceOf::<T>::from(fiat_amount_cents / cents_per_token))
				.ok_or(ArithmeticError::Overflow)?;

			ComputeHours::<T>::mutate(&account, |hours| *hours += compute_hours as u32);
			StripePayments::<T>::insert(&bounded_payment_id, account.clone());

			let provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;
			T::Currency::transfer(
				&provider,
				&account,
				native_value,
				ExistenceRequirement::KeepAlive,
			)?;

			Self::deposit_event(Event::FiatPaymentProcessed(
				account.clone(),
				compute_hours as u32,
			));
			Self::deposit_event(Event::ConsumerSubscribed(
				account.clone(),
				native_value,
				compute_hours as u32,
			));

			Ok(())
		}

		/// Create a FIAT payout request for a miner
		#[pallet::call_index(11)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::request_fiat_payout())]
		pub fn request_fiat_payout(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let miner = ensure_signed(origin)?;
			ensure!(
				pallet_edge_connect::Pallet::<T>::is_registered_miner(&miner),
				Error::<T>::NotRegisteredMiner
			);

			MinerFiatPayouts::<T>::mutate(&miner, |pending| *pending += amount);
			Self::deposit_event(Event::MinerFiatPayoutCreated(miner, amount));

			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::get_remaining_hours())]
		pub fn get_remaining_hours(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let hours = ComputeHours::<T>::get(&who);
			Self::deposit_event(Event::RemainingHoursQueried(who, hours));
			Ok(())
		}
	}
}
