#![cfg_attr(not(feature = "std"), no_std)]

use cyborg_primitives::payment::*;
use frame_support::{sp_runtime::traits::AccountIdConversion, PalletId};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::traits::tokens::Preservation;
	use frame_support::traits::EnsureOriginWithArg;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::Saturating,
		traits::{tokens::fungibles::{Inspect as FungiblesInspect, Mutate as FungiblesMutate}, Currency, ExistenceRequirement, ReservableCurrency},
	};
	use sp_runtime::traits::AtLeast32BitUnsigned;
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

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_edge_connect::Config + scale_info::TypeInfo
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: ReservableCurrency<Self::AccountId>;

		// type WeightInfo: WeightInfo;
		// type TreasuryAccount: Get<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

        #[pallet::constant]
		type MaxKycHashLength: Get<u32>;

		#[pallet::constant]
		type SubscriptionPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type OnDemandPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
        type MaxUserIdLength: Get<u32>;

        #[pallet::constant]
		type OnDemandRate: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type SubscriptionRate: Get<BalanceOf<Self>>;

		/// Asset registry for multi-asset support
		type AssetRegistry: FungiblesInspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ FungiblesMutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>;

		/// Asset ID type
		type AssetId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ From<u32>;

		/// Asset Balance type
		type AssetBalance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ TypeInfo
			+ TryFrom<BalanceOf<Self>>
			+ Into<BalanceOf<Self>>;

		/// Ensure origin for asset operations
		type AssetAuthority: EnsureOriginWithArg<Self::RuntimeOrigin, Self::AssetId>;
	}

	#[pallet::storage]
	pub type ActivePayments<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
        Blake2_128Concat,
        PaymentPurpose,
		PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>,
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
	pub(super) type SubscriptionFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// storage that tracks compute hours for all users regardless of payment method
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

	/// Storage for asset-based subscription fees per hour
	#[pallet::storage]
	pub type AssetSubscriptionFees<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetBalance, OptionQuery>;

	/// Event declarations for extrinsic calls.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PaymentExpired(T::AccountId, PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>),
		PaymentToppedUp {
			account: T::AccountId,
			details: PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>,
			old_expiry: BlockNumberFor<T>,
		},
		PaymentReservedRelease {
			account: T::AccountId,
			details: PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>,
			released: BlockNumberFor<T>,
		},
        TaskCashBack {
            account: T::AccountId,
            purpose: PaymentPurpose,
            details: PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>,
            amount: BalanceOf<T>,
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

		/// When admin sets subscription fee for a specific asset
		AssetSubscriptionFeeSet {
			asset_id: T::AssetId,
			fee_per_hour: T::AssetBalance,
		},
		/// When user subscribes using a specific asset
		AssetSubscribed {
			account: T::AccountId,
			asset_id: T::AssetId,
			total_fee: T::AssetBalance,
			hours: u32,
		},
		/// When user adds hours using a specific asset
		AssetHoursAdded {
			account: T::AccountId,
			asset_id: T::AssetId,
			extra_hours: u32,
			total_fee: T::AssetBalance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		PaymentAlreadyActive, // Payment is already active (begin and expiry set)
		NoActivePayments,
		InvalidPaymentMode,
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
		/// Asset subscription fee not set for this asset
		AssetFeeNotSet,
		/// Insufficient asset balance
		InsufficientAssetBalance,
		/// Invalid asset ID
		InvalidAssetId,
		/// Asset transfer failed
		AssetTransferFailed,
		/// Balance conversion failed
		BalanceConversionFailed,
	}

	impl<T: Config> Pallet<T> {
		/// Get the pallet's account ID
		pub fn pallet_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn reserve(
			who: &T::AccountId,
			mode: PaymentMode,
			purpose: PaymentPurpose,
		) -> Result<PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>, DispatchError> {
            let pallet_account = Self::pallet_account_id();

			let amount = match mode {
				PaymentMode::OnDemand => T::OnDemandRate::get(),
				PaymentMode::Subscription => T::SubscriptionRate::get(),
			};

			T::Currency::transfer(who, &pallet_account, amount, ExistenceRequirement::KeepAlive)?;

			T::Currency::reserve(&pallet_account, amount)?;

			let details = PaymentDetails {
				begin: Default::default(),
				expiry: Default::default(),
				reserved: amount,
				mode,
			};

            ActivePayments::<T>::insert(who, purpose, details.clone());

			Ok(details)
		}

		pub fn unreserve(
			task_owner: &T::AccountId,
            purpose: PaymentPurpose,
			mut details: PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>,
		) -> Result<PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>, DispatchError> {
            
            if Self::has_active_payment(task_owner, purpose.clone()) {
                return Err(Error::<T>::PaymentAlreadyActive.into());
            }

			let now = frame_system::Pallet::<T>::block_number();

			let pallet_account = Self::pallet_account_id();

			details.begin = now;
			details.expiry = match details.mode {
				PaymentMode::OnDemand => now.saturating_add(T::OnDemandPeriod::get()),
				PaymentMode::Subscription => now.saturating_add(T::SubscriptionPeriod::get()),
			};

			T::Currency::unreserve(&pallet_account, details.reserved);

			ActivePayments::<T>::insert(task_owner, purpose, details.clone());

			Self::deposit_event(Event::PaymentReservedRelease {
				account: task_owner.clone(),
				details: details.clone(),
				released: now,
			});

			Ok(details)
		}

		pub fn release(
			who: &T::AccountId,
			purpose: PaymentPurpose,
		) -> DispatchResult {
			let details = ActivePayments::<T>::take(who, purpose.clone()).ok_or(Error::<T>::NoActivePayments)?;

            let treasury = Self::pallet_account_id();

			T::Currency::unreserve(&treasury, details.reserved);

			// Return funds to user
			T::Currency::transfer(
				&treasury,
				who,
				details.reserved,
                ExistenceRequirement::KeepAlive,
            )?;
			/*
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
			*/
            Ok(())
		}

        /*
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
				<T as pallet::Config>::Currency::transfer(
					&provider,
					&miner,
					reward,
					ExistenceRequirement::KeepAlive,
				)?;
				info!("{:?} rewarded with {:?} Native Coin", miner, reward);
				Self::deposit_event(Event::MinerRewarded(miner, reward));
			}
			Ok(())
		}

		/// Allows a new user to subscribe to compute by paying upfront with native currency.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::subscribe())]
		pub fn subscribe(origin: OriginFor<T>, hours: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(hours > 0, Error::<T>::InvalidHoursInput);

			// Check if user already has compute hours (is already subscribed)
			let current_hours = ComputeHours::<T>::get(&who);
			ensure!(current_hours == 0, Error::<T>::AlreadySubscribed);

			let fee_per_hour = SubscriptionFee::<T>::get();
			let total_fee = fee_per_hour
				.checked_mul(&hours.into())
				.ok_or(Error::<T>::InvalidFee)?;
			ensure!(
				<T as pallet::Config>::Currency::free_balance(&who) >= total_fee,
				Error::<T>::InsufficientBalance
			);
			let provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;
			<T as pallet::Config>::Currency::transfer(
				&who,
				&provider,
				total_fee,
				ExistenceRequirement::KeepAlive,
			)?;
			ComputeHours::<T>::mutate(&who, |current_hours| *current_hours += hours);
			Self::deposit_event(Event::ConsumerSubscribed(who, total_fee, hours));
			Ok(())
		}

		/// Lets an existing user add more hours to their subscription using native currency.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_hours())]
		pub fn add_hours(origin: OriginFor<T>, extra_hours: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(extra_hours > 0, Error::<T>::InvalidHoursInput);

			// Check if user has an active subscription (has compute hours)
			let current_hours = ComputeHours::<T>::get(&who);
			ensure!(current_hours > 0, Error::<T>::SubscriptionExpired);

			let fee_per_hour = SubscriptionFee::<T>::get();
			let total_fee = fee_per_hour
				.checked_mul(&extra_hours.into())
				.ok_or(Error::<T>::InvalidFee)?;
			ensure!(
				<T as pallet::Config>::Currency::free_balance(&who) >= total_fee,
				Error::<T>::InsufficientBalance
			);
			let provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;
			<T as pallet::Config>::Currency::transfer(
				&who,
				&provider,
				total_fee,
				ExistenceRequirement::KeepAlive,
			)?;
			ComputeHours::<T>::mutate(&who, |hours| {
				*hours += extra_hours;
			});
			Self::deposit_event(Event::SubscriptionRenewed(who, extra_hours));
			Ok(())
		}

		/// Admin sets the global subscription cost per compute hour for native currency.
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
			<T as pallet::Config>::Currency::transfer(
				&provider,
				&account,
				native_value,
>>>>>>> dev
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(())
		}
        */

		pub fn cashback(who: &T::AccountId, purpose: PaymentPurpose, amount: BalanceOf<T>) -> DispatchResult {
			let details = ActivePayments::<T>::take(who, purpose.clone()).ok_or(Error::<T>::NoActivePayments)?;

			let pallet_account = Self::pallet_account_id();

			// Unreserve the refund amount from pallet account
			T::Currency::unreserve(&pallet_account, amount);

			// Transfer refund to user
			T::Currency::transfer(&pallet_account, who, amount, ExistenceRequirement::KeepAlive)?;

			Self::deposit_event(Event::TaskCashBack {
				account: who.clone(),
                purpose,
                details,
				amount,
			});

			Ok(())
		}

		pub fn has_active_payment(who: &T::AccountId, purpose: PaymentPurpose) -> bool {
			let now = frame_system::Pallet::<T>::block_number();

            if let Some(details) = ActivePayments::<T>::get(who, purpose) {
                // Only consider payments that have been properly activated
        
                if details.begin == Zero::zero() && details.expiry == Zero::zero() {
                    return false;
                }
                match details.mode {
                    PaymentMode::Subscription =>
                        now <= details.expiry.saturating_add(T::GracePeriod::get()),
                    PaymentMode::OnDemand => now <= details.expiry,
                }
            } else {
                false
            }
        }
        // TODO: Optionally, claim_reward() & cancel_reward(Admin)
        // I much prefer rewards be handled as session management for miner(i.e collator)

        // TODO: Create a view function to see the current running time for each task
        // TODO: Create a view function to see active miner rewards.	
    }

        /*

		/// Get remaining hours from the unified compute hours pool
		#[pallet::call_index(14)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::get_remaining_hours())]
		pub fn get_remaining_hours(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let hours = ComputeHours::<T>::get(&who);
			Self::deposit_event(Event::RemainingHoursQueried(who, hours));
			Ok(())
		}

		/// Admin sets subscription fee for a specific asset
		#[pallet::call_index(15)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_asset_subscription_fee())]
		pub fn set_asset_subscription_fee(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			fee_per_hour: T::AssetBalance,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(fee_per_hour > Zero::zero(), Error::<T>::InvalidFee);

			// Verify asset exists by checking if we can get its total issuance
			let _total_issuance = T::AssetRegistry::total_issuance(asset_id);

			AssetSubscriptionFees::<T>::insert(asset_id, fee_per_hour);

			Self::deposit_event(Event::AssetSubscriptionFeeSet {
				asset_id,
				fee_per_hour,
			});

			Ok(())
		}

		/// Subscribe using a specific asset
		#[pallet::call_index(16)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::subscribe_with_asset())]
		pub fn subscribe_with_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			hours: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(hours > 0, Error::<T>::InvalidHoursInput);

			// Check if user already has compute hours (is already subscribed)
			let current_hours = ComputeHours::<T>::get(&who);
			ensure!(current_hours == 0, Error::<T>::AlreadySubscribed);

			// Get fee for this asset
			let fee_per_hour =
				AssetSubscriptionFees::<T>::get(asset_id).ok_or(Error::<T>::AssetFeeNotSet)?;

			let total_fee = fee_per_hour
				.checked_mul(&hours.into())
				.ok_or(ArithmeticError::Overflow)?;

			// Check asset balance
			let asset_balance = T::AssetRegistry::balance(asset_id, &who);
			ensure!(
				asset_balance >= total_fee,
				Error::<T>::InsufficientAssetBalance
			);

			let provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;

			// Transfer assets
			T::AssetRegistry::transfer(asset_id, &who, &provider, total_fee, Preservation::Preserve)?;

			// Add compute hours to the unified pool
			ComputeHours::<T>::mutate(&who, |current_hours| *current_hours += hours);

			Self::deposit_event(Event::AssetSubscribed {
				account: who,
				asset_id,
				total_fee,
				hours,
			});

			Ok(())
		}

		/// Add hours using a specific asset
		#[pallet::call_index(17)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_hours_with_asset())]
		pub fn add_hours_with_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			extra_hours: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(extra_hours > 0, Error::<T>::InvalidHoursInput);

			// Check if user has an active subscription (has compute hours)
			let current_hours = ComputeHours::<T>::get(&who);
			ensure!(current_hours > 0, Error::<T>::SubscriptionExpired);

			// Get fee for this asset
			let fee_per_hour =
				AssetSubscriptionFees::<T>::get(asset_id).ok_or(Error::<T>::AssetFeeNotSet)?;

			let total_fee = fee_per_hour
				.checked_mul(&extra_hours.into())
				.ok_or(ArithmeticError::Overflow)?;

			// Check asset balance
			let asset_balance = T::AssetRegistry::balance(asset_id, &who);
			ensure!(
				asset_balance >= total_fee,
				Error::<T>::InsufficientAssetBalance
			);

			let provider =
				ServiceProviderAccount::<T>::get().ok_or(Error::<T>::ServiceProviderAccountNotFound)?;

			// Transfer assets
			T::AssetRegistry::transfer(asset_id, &who, &provider, total_fee, Preservation::Preserve)?;

			// Add compute hours to the unified pool
			ComputeHours::<T>::mutate(&who, |hours| {
				*hours += extra_hours;
			});

			Self::deposit_event(Event::AssetHoursAdded {
				account: who,
				asset_id,
				extra_hours,
				total_fee,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get total compute hours for a user (now unified)
		pub fn get_total_compute_hours(account: &T::AccountId) -> u32 {
			ComputeHours::<T>::get(account)
		}

		/// Check if user has sufficient compute hours
		pub fn has_sufficient_hours(account: &T::AccountId, required_hours: u32) -> bool {
			Self::get_total_compute_hours(account) >= required_hours
		}
	}
*/
}
