#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::*;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;

pub mod weights;
use cyborg_primitives::payment::RewardRates;
use log::info;

pub use weights::*;
pub use types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{traits::CheckedMul, ArithmeticError, Saturating},
		traits::{ReservableCurrency, Currency, ExistenceRequirement},
	};
	use sp_std::vec::Vec;

	use super::*;

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
		#[pallet::constant]
		type MaxKycHashLength: Get<u32>;

		/// Maximum length for user IDs
		#[pallet::constant]
		type MaxUserIdLength: Get<u32>;

		/// Maximum length for payment IDs
		#[pallet::constant]
		type MaxPaymentIdLength: Get<u32>;
        
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

	/// Storage for mapping Stripe payment IDs to on-chain accounts
	#[pallet::storage]
	pub type StripePayments<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxPaymentIdLength>,
		T::AccountId,
		OptionQuery,
	>;

	/// Storage for all user KYC information
	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub type Users<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, UserInfo<T>, OptionQuery>;

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
	pub(super) type SubscriptionFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>; // TODO: Deprecate in favour of ActivePayments.

	/// Storage map that tracks the number of compute hours owned by each account.
	#[pallet::storage]
	pub type ComputeHours<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>; // TODO: Deprecate in favour of ActivePayments.

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

    #[pallet::storage]
    pub type ActivePayments<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, PaymentMode, PaymentPeriod<BlockNumberFor<T>>>;

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

		/// When a user submits KYC documents
		KycSubmitted {
			account: T::AccountId,
			user_id: BoundedVec<u8, T::MaxUserIdLength>,
			document_hash: BoundedVec<u8, T::MaxKycHashLength>,
		},
		/// When KYC is verified
		KycVerified {
			account: T::AccountId,
			user_id: BoundedVec<u8, T::MaxUserIdLength>,
			// verification_hash: BoundedVec<u8, T::MaxKycHashLength>,
			verified_at: BlockNumberFor<T>,
		},
		/// When KYC is rejected
		KycRejected {
			account: T::AccountId,
			user_id: BoundedVec<u8, T::MaxUserIdLength>,
			// reason: BoundedVec<u8, T::MaxKycHashLength>,
		},
		FiatPaymentProcessed(T::AccountId, u32), // Account and compute hours added
		MinerFiatPayoutCreated(T::AccountId, BalanceOf<T>), // Miner payout record created
		FiatConversionRateUpdated(u64, BalanceOf<T>), // Rate updated (cents per native token)
		RemainingHoursQueried(T::AccountId, u32),
        PaymentActivated{
            account: T::AccountId,
            mode: PaymentMode,
            period: PaymentPeriod<BlockNumberFor<T>>,
        },
        HasActivePayment(T::AccountId),
        PaymentExpired(T::AccountId, PaymentMode),
        ExpiredPaymentsCleaned(u32),
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
		/// KYC verification hash is too long
		KycHashTooLong,
		/// KYC verification already exists for this account
		KycAlreadyVerified,
		/// User ID is too long
		UserIdTooLong,
		KycNotSubmitted,
		KycVerificationPending,
		/// Rejection reason is too long
		KycRejected,
		InvalidStripePaymentId,
		FiatConversionRateNotSet,
        PaymentAlreadyActive,
	}



     /*
     // Add to your pallet's hooks implementation
     #[pallet::hooks]
     impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
         fn on_initialize(_n: BlockNumberFor<T>) -> Weight {             
             // Clean expired payments on every block initialization
             Self::clean_expired_payments();

             // Return actual weight measurement in production
             T::DbWeight::get().reads_writes(1, 1)
         }
     }
     */

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
				pallet_edge_connect::Pallet::<T>::account_miners(&who).is_some(),
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

		/// Users submit their KYC documents
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::submit_kyc())]
		pub fn submit_kyc(
			origin: OriginFor<T>,
			user_id: Vec<u8>,
			document_hash: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_user_id = BoundedVec::try_from(user_id).map_err(|_| Error::<T>::UserIdTooLong)?;
			let bounded_hash =
				BoundedVec::try_from(document_hash).map_err(|_| Error::<T>::KycHashTooLong)?;

			// Check if user already exists
			if let Some(user_info) = Users::<T>::get(&who) {
				match user_info.status {
					VerificationStatus::Verified(_) => return Err(Error::<T>::KycAlreadyVerified.into()),
					VerificationStatus::Pending => return Err(Error::<T>::KycVerificationPending.into()),
					VerificationStatus::Rejected => {
						// Allow resubmission if previously rejected
						let new_user_info = UserInfo {
							user_id: bounded_user_id.clone(),
							document_hash: bounded_hash.clone(),
							status: VerificationStatus::Pending,
						};
						Users::<T>::insert(&who, new_user_info);
					}
				}
			} else {
				// New submission
				let user_info = UserInfo {
					user_id: bounded_user_id.clone(),
					document_hash: bounded_hash.clone(),
					status: VerificationStatus::Pending,
				};
				Users::<T>::insert(&who, user_info);
			}

			Self::deposit_event(Event::KycSubmitted {
				account: who,
				user_id: bounded_user_id,
				document_hash: bounded_hash,
			});

			Ok(())
		}

		/// Admin verifies KYC submission
		#[pallet::call_index(10)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::verify_kyc())]
		pub fn verify_kyc(
			origin: OriginFor<T>,
			account: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			ensure_root(origin)?;

			let mut user_info = Users::<T>::get(&account).ok_or(Error::<T>::KycNotSubmitted)?;

			if approved {
				user_info.status = VerificationStatus::Verified(frame_system::Pallet::<T>::block_number());
				Users::<T>::insert(&account, &user_info);

				Self::deposit_event(Event::KycVerified {
					account,
					user_id: user_info.user_id.clone(),
					verified_at: frame_system::Pallet::<T>::block_number(),
				});
			} else {
				user_info.status = VerificationStatus::Rejected;
				Users::<T>::insert(&account, &user_info);

				Self::deposit_event(Event::KycRejected {
					account,
					user_id: user_info.user_id.clone(),
				});
			}

			Ok(())
		}

		/// Admin sets the conversion rate between FIAT (cents) and native tokens
		#[pallet::call_index(11)]
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
		#[pallet::call_index(12)]
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
				.checked_mul(
					&BalanceOf::<T>::try_from(fiat_amount_cents / cents_per_token)
						.map_err(|_| ArithmeticError::Overflow)?,
				)
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
		#[pallet::call_index(13)]
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

		#[pallet::call_index(14)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::get_remaining_hours())]
		pub fn get_remaining_hours(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let hours = ComputeHours::<T>::get(&who);
			Self::deposit_event(Event::RemainingHoursQueried(who, hours));
			Ok(())
		}

        ///
        /// Activate Payments for Compute Consumption.
        #[pallet::call_index(15)]
        #[pallet::weight(0)]
        pub fn activate(
            origin: OriginFor<T>,
            mode: PaymentMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!ActivePayments::<T>::contains_key(&who, &mode), Error::<T>::PaymentAlreadyActive);

            let on_demand_rate = T::OnDemandRate::get();

            let subscription_rate = T::SubscriptionRate::get();

            let on_demand_period = T::OnDemandPeriod::get();

            let subscription_period = T::SubscriptionPeriod::get();

            let current_block = frame_system::Pallet::<T>::block_number();

            let provider = ServiceProviderAccount::<T>::get().ok_or(Error::<T>::SubscriptionExpired)?;

            let (start_block, end_block) = match mode {
                PaymentMode::OnDemand => {
                    ensure!(T::Currency::free_balance(&who) > on_demand_rate, Error::<T>::InsufficientBalance);

                    T::Currency::transfer(&who, &provider, on_demand_rate, ExistenceRequirement::KeepAlive)?;

                    (current_block, current_block.saturating_add(on_demand_period))
                },
                PaymentMode::Subscription => {
                    ensure!(T::Currency::free_balance(&who) > subscription_rate, Error::<T>::InsufficientBalance);

                    T::Currency::transfer(&who, &provider, subscription_rate, ExistenceRequirement::KeepAlive)?;

                    (current_block, current_block.saturating_add(subscription_period))
                },
            };

            let payment_period = PaymentPeriod {
                start_block,
                end_block,
            };

            ActivePayments::<T>::insert(who.clone(), mode.clone(), payment_period.clone());

            Self::deposit_event(Event::PaymentActivated {
                account: who,
                mode: mode,
                period: payment_period
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn has_active_payment(who: &T::AccountId) -> DispatchResult {

            // Check if user has an active payment
            let has_active_payment = Self::check_and_clean_user_payments(who);

            // Ensure user has either an active subscription or on-demand payment
            ensure!(has_active_payment, Error::<T>::InsufficientComputeHours);

            // Emit the event
            Self::deposit_event(Event::HasActivePayment(who.clone()));

            Ok(())
        }

        pub fn check_and_clean_user_payments(who: &T::AccountId) -> bool {
            let current_block = frame_system::Pallet::<T>::block_number();
            let mut has_active_payment = false;

            // Check and clean subscription
            if let Some(subscription_period) = ActivePayments::<T>::get(who, PaymentMode::Subscription) {
                if current_block <= subscription_period.end_block {
                    has_active_payment = true;
                } else {
                    // Clean expired subscription
                    ActivePayments::<T>::remove(who, PaymentMode::Subscription);
                    Self::deposit_event(Event::PaymentExpired(who.clone(), PaymentMode::Subscription));
                }
            }

            // Check and clean on-demand payment if no active subscription
            if !has_active_payment {
                if let Some(on_demand_period) = ActivePayments::<T>::get(who, PaymentMode::OnDemand) {
                    if current_block <= on_demand_period.end_block {
                        has_active_payment = true;
                    } else {
                        // Clean expired on-demand payment
                        ActivePayments::<T>::remove(who, PaymentMode::OnDemand);
                        Self::deposit_event(Event::PaymentExpired(who.clone(), PaymentMode::OnDemand));
                    }
                }
            }

            has_active_payment
        }

        /*

        /// Clean all expired payments across all users
        pub fn clean_expired_payments() {
            let current_block = frame_system::Pallet::<T>::block_number();
            let mut cleaned_count = 0u32;

            // Iterate through all active payments and remove expired ones
            // Note: This might be heavy - consider using a bounded iteration or migration pattern
            // for production use with many users
            ActivePayments::<T>::iter().for_each(|(who, payment_mode, period)| {
                if current_block > period.end_block {
                    ActivePayments::<T>::remove(&who, &payment_mode);
                    Self::deposit_event(Event::PaymentExpired(who, payment_mode));
                    cleaned_count += 1;
                }
            });

            // Emit event if any cleanups occurred
            if cleaned_count > 0 {
                Self::deposit_event(Event::ExpiredPaymentsCleaned(cleaned_count));
            }
        }
        */
    }
}
