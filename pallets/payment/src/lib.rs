#![cfg_attr(not(feature = "std"), no_std)]

use cyborg_primitives::payment::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
use cyborg_primitives::payment::RewardRates;
use log::info;

pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{traits::CheckedMul, ArithmeticError, Saturating},
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
	};
	use sp_std::vec::Vec;

	use super::*;

    /*
	#[derive(Encode, Decode, RuntimeDebug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub enum VerificationStatus<BlockNumber> {
		Pending,
		Verified(BlockNumber),
		Rejected,
	}

	#[derive(Encode, Decode, RuntimeDebug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub struct UserInfo<T: Config> {
		user_id: BoundedVec<u8, T::MaxUserIdLength>,
		document_hash: BoundedVec<u8, T::MaxKycHashLength>,
		status: VerificationStatus<BlockNumberFor<T>>,
	}

	impl<T: Config> Clone for UserInfo<T> {
		fn clone(&self) -> Self {
			Self {
				user_id: self.user_id.clone(),
				document_hash: self.document_hash.clone(),
				status: self.status.clone(),
			}
		}
	}

	impl<BlockNumber: Clone> Clone for VerificationStatus<BlockNumber> {
		fn clone(&self) -> Self {
			match self {
				VerificationStatus::Pending => VerificationStatus::Pending,
				VerificationStatus::Verified(block_num) => VerificationStatus::Verified(block_num.clone()),
				VerificationStatus::Rejected => VerificationStatus::Rejected,
			}
		}
	}
    */

	/// Type alias to simplify balance-related operations.
	/// This maps the `Balance` type based on the associated `Currency` in the runtime config.
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_edge_connect::Config + scale_info::TypeInfo
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Abstraction over the chain's currency system, allowing this pallet to interact with balances.
		type Currency: ReservableCurrency<Self::AccountId>;

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

		/// Maximum length for KYC verification hash
		//#[pallet::constant]
		//type MaxKycHashLength: Get<u32>;

		/// Maximum length for user IDs
		//#[pallet::constant]
		//type MaxUserIdLength: Get<u32>;

		/// Maximum length for payment IDs
		//#[pallet::constant]
		//type MaxPaymentIdLength: Get<u32>;

		#[pallet::constant]
		type SubscriptionPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type OnDemandPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type OnDemandRate: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type SubscriptionRate: Get<BalanceOf<Self>>;
	}

	/// Storage that holds the service provider's account ID.
	#[pallet::storage]
	pub type ServiceProviderAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	/// Store the latest recorded usage (cpu%, ram%, storage%) for miners.
	//#[pallet::storage]
	//pub type MinerUsage<T: Config> =
	//	StorageMap<_, Blake2_128Concat, T::AccountId, (u8, u8, u8), OptionQuery>; // cpu, ram, storage usage percentages

	/// Store rewards waiting to be distributed to miners.
	//#[pallet::storage]
	//pub type MinerPendingRewards<T: Config> =
	//	StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Store custom reward rates when miner is active.
	//#[pallet::storage]
	//#[pallet::getter(fn active_reward_rates)]
	//pub type ActiveRewardRates<T: Config> =
		//StorageMap<_, Blake2_128Concat, T::AccountId, RewardRates<BalanceOf<T>>, OptionQuery>;

	/// Store custom reward rates when miner is idle.
	//#[pallet::storage]
	//#[pallet::getter(fn idle_reward_rates)]
	//pub type IdleRewardRates<T: Config> =
	//StorageMap<_, Blake2_128Concat, T::AccountId, RewardRates<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	pub type ActivePayments<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		PaymentMode,
		PaymentPeriod<BlockNumberFor<T>>,
	>;

	/// Event declarations for extrinsic calls.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ServiceProviderAccountSet(T::AccountId), // When admin sets provider.
		MinerRewarded(T::AccountId, BalanceOf<T>), // Reward given to a miner.
		PaymentExpired(T::AccountId, PaymentMode),
		PaymentToppedUp {
			account: T::AccountId,
			mode: PaymentMode,
			extended_from: BlockNumberFor<T>,
			extended_to: BlockNumberFor<T>,
		},
	}

	/// Custom pallet errors.
	#[pallet::error]
	pub enum Error<T> {
		ServiceProviderAccountNotFound, // Service provider isn't set.
		InvalidUsageInput,              // Usage percentages out of bounds.
		PaymentAlreadyActive,
		NoActivePayments,
		InvalidPaymentMode,
	}

	/// Declare callable extrinsics.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
			TryFrom<u64>,
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

        /*

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
        */
	}

	impl<T: Config> Pallet<T> {
        pub fn activate(
            who: &T::AccountId, 
            mode: PaymentMode, 
            purpose: PaymentPurpose
        ) -> Result<PaymentPeriod<BlockNumberFor<T>>, DispatchError> {
            let current_block = frame_system::Pallet::<T>::block_number();


            if Self::check_and_clean_user_payments(who).0 {
                return Err(Error::<T>::PaymentAlreadyActive.into());
            }

            let provider =
                ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;

            let (start_block, end_block) = match mode {
                PaymentMode::OnDemand => {
                    T::Currency::transfer(
                        &who,
                        &provider,
                        T::OnDemandRate::get(),
                        ExistenceRequirement::KeepAlive,
                    )?;
                    
                    (
                        current_block,
                        current_block.saturating_add(T::OnDemandPeriod::get()),
                    )
                }
                PaymentMode::Subscription => {
                    T::Currency::transfer(
                        &who,
                        &provider,
                        T::SubscriptionRate::get(),
                        ExistenceRequirement::KeepAlive,
                    )?;

                    (
                        current_block,
                        current_block.saturating_add(T::SubscriptionPeriod::get()),
                    )
                }
            };

            let payment_period = PaymentPeriod {
                start_block,
                end_block,
                purpose,
            };

            ActivePayments::<T>::insert(who.clone(), mode.clone(), payment_period.clone());

            Ok(payment_period)
        }

        pub fn top_up(who: &T::AccountId, mode: PaymentMode) -> DispatchResult {
            let provider =
                ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;
            let mut new_end_block: BlockNumberFor<T> = Zero::zero();
            let mut previous_end_block: BlockNumberFor<T> = Zero::zero();

            // Check and clean user payments
            let (has_active_payment, active_modes) = Self::check_and_clean_user_payments(&who);

            // Ensure user has an active payment
            ensure!(has_active_payment, Error::<T>::NoActivePayments);

            // Ensure the requested payment mode is actually active
            ensure!(active_modes.contains(&mode), Error::<T>::InvalidPaymentMode);

            match mode {
                PaymentMode::OnDemand => {
                    T::Currency::transfer(
                        &who,
                        &provider,
                        T::OnDemandRate::get(),
                        ExistenceRequirement::KeepAlive,
                    )?;

                    ActivePayments::<T>::mutate(&who, &mode, |existing_period| {
                        if let Some(period) = existing_period {
                            // Extend the end block by the extension period
                            previous_end_block = period.end_block;
                            new_end_block = period.end_block.saturating_add(T::OnDemandPeriod::get());
                            period.end_block = new_end_block;
                        }
                    });
                }

                PaymentMode::Subscription => {
                    T::Currency::transfer(
                        &who,
                        &provider,
                        T::SubscriptionRate::get(),
                        ExistenceRequirement::KeepAlive,
                    )?;

                    ActivePayments::<T>::mutate(&who, &mode, |existing_period| {
                        if let Some(period) = existing_period {
                            // Extend the end block by the extension period
                            previous_end_block = period.end_block;
                            new_end_block = period
                                .end_block
                                .saturating_add(T::SubscriptionPeriod::get());
                            period.end_block = new_end_block;
                        }
                    });
                }
            };

            Self::deposit_event(Event::PaymentToppedUp {
                account: who.clone(),
                mode,
                extended_from: previous_end_block,
                extended_to: new_end_block,
            });
            Ok(())
        }

		pub fn check_and_clean_user_payments(who: &T::AccountId) -> (bool, Vec<PaymentMode>) {
			let current_block = frame_system::Pallet::<T>::block_number();
			let mut active_modes = Vec::new();

			// Check and clean subscription
			if let Some(subscription_period) = ActivePayments::<T>::get(who, PaymentMode::Subscription) {
				if current_block
					<= subscription_period
						.end_block
						.saturating_add(T::GracePeriod::get())
				{
					active_modes.push(PaymentMode::Subscription);
				} else {
					// Clean expired subscription
					ActivePayments::<T>::remove(who, PaymentMode::Subscription);
					Self::deposit_event(Event::PaymentExpired(
						who.clone(),
						PaymentMode::Subscription,
					));
				}
			}

			// Check and clean on-demand payment if no active subscription
			if let Some(on_demand_period) = ActivePayments::<T>::get(who, PaymentMode::OnDemand) {
				if current_block <= on_demand_period.end_block {
					active_modes.push(PaymentMode::OnDemand);
				} else {
					// Clean expired on-demand payment
					ActivePayments::<T>::remove(who, PaymentMode::OnDemand);
					Self::deposit_event(Event::PaymentExpired(who.clone(), PaymentMode::OnDemand));
				}
			}

			let has_active_payment = !active_modes.is_empty();
			(has_active_payment, active_modes)
		}
	}
}
