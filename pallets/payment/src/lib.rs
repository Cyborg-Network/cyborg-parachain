#![cfg_attr(not(feature = "std"), no_std)]

use cyborg_primitives::payment::*;
use frame_support::{sp_runtime::traits::AccountIdConversion, PalletId};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		require_transactional,
		sp_runtime::Saturating,
		traits::{
			tokens::{
				fungibles::{Inspect as FungiblesInspect, Mutate as FungiblesMutate},
				Preservation,
			},
			Currency, EnsureOriginWithArg, ExistenceRequirement, ReservableCurrency,
		},
	};
	use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
	use sp_runtime::traits::AtLeast32BitUnsigned;
	use sp_std::vec::Vec;

	use super::*;

	pub type BalanceOf<T> =
		<<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type AssetIdOf<T> =
		<<T as Config>::Asset as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub type PaymentDetailsOf<T> = PaymentDetails<BlockNumberFor<T>, AssetIdOf<T>, BalanceOf<T>>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_edge_connect::Config + scale_info::TypeInfo
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type SubscriptionPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type OnDemandPeriod: Get<BlockNumberFor<Self>>;

		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;

		type Rate: PaymentRates<AssetIdOf<Self>, BalanceOf<Self>>;

		type Asset: MultiReservableCurrency<Self::AccountId>;
	}

	#[pallet::storage]
	pub type ActivePayments<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		PaymentPurpose,
		PaymentDetailsOf<T>,
	>;

	/// Store custom reward rates when miner is active.
	#[pallet::storage]
	#[pallet::getter(fn active_reward_rates)]
	pub type ActiveRewardRates<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RewardRates<BalanceOf<T>>, OptionQuery>;

	/// Event declarations for extrinsic calls.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PaymentExpired(T::AccountId, PaymentDetailsOf<T>),
		PaymentToppedUp {
			account: T::AccountId,
			details: PaymentDetailsOf<T>,
			old_expiry: BlockNumberFor<T>,
		},
		PaymentReservedRelease {
			account: T::AccountId,
			details: PaymentDetailsOf<T>,
			released: BlockNumberFor<T>,
		},
		TaskCashBack {
			account: T::AccountId,
			purpose: PaymentPurpose,
			details: PaymentDetailsOf<T>,
			amount: BalanceOf<T>,
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
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		#[require_transactional]
		pub fn reserve(
			who: &T::AccountId,
			purpose: PaymentPurpose,
			mut details: PaymentDetailsOf<T>,
		) -> Result<PaymentDetailsOf<T>, DispatchError> {
			let escrow = Self::account_id();

			let amount = T::Rate::get_rate(details.asset.clone(), details.mode.clone());

			details.amount = amount;

			T::Asset::reserve(details.asset.clone(), who, details.amount.clone())?;

			T::Asset::repatriate_reserved(
				details.asset.clone(),
				who,
				&escrow,
				details.amount.clone(),
				BalanceStatus::Reserved,
			)?;

			ActivePayments::<T>::insert(who, purpose, details.clone());

			Ok(details)
		}

		pub fn set_active(
			task_owner: &T::AccountId,
			purpose: PaymentPurpose,
			mut details: PaymentDetailsOf<T>,
		) -> Result<PaymentDetailsOf<T>, DispatchError> {
			if Self::has_active_payment(task_owner, purpose.clone()) {
				return Err(Error::<T>::PaymentAlreadyActive.into());
			}

			let now = frame_system::Pallet::<T>::block_number();

			details.begin = now;
			details.expiry = match details.mode {
				PaymentMode::OnDemand => now.saturating_add(T::OnDemandPeriod::get()),
				PaymentMode::Subscription => now.saturating_add(T::SubscriptionPeriod::get()),
			};

			ActivePayments::<T>::insert(task_owner, purpose, details.clone());

			Self::deposit_event(Event::PaymentReservedRelease {
				account: task_owner.clone(),
				details: details.clone(),
				released: now,
			});

			Ok(details)
		}

		#[require_transactional]
		pub fn release(who: &T::AccountId, purpose: PaymentPurpose) -> DispatchResult {
			let details = ActivePayments::<T>::take(who, purpose.clone())
				.ok_or(Error::<T>::NoActivePayments)?;

			let escrow = Self::account_id();

			T::Asset::unreserve(details.asset, &escrow, details.amount);

			// Return funds to user
			T::Asset::transfer(
				details.asset,
				who,
				&escrow,
				details.amount,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(())
		}

		#[require_transactional]
		pub fn cashback(
			who: &T::AccountId,
			purpose: PaymentPurpose,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let details = ActivePayments::<T>::take(who, purpose.clone())
				.ok_or(Error::<T>::NoActivePayments)?;

			let escrow = Self::account_id();

			// Unreserve the refund amount from pallet account
			T::Asset::unreserve(details.asset, &escrow, amount);

			// Transfer cashback to user
			T::Asset::transfer(
				details.asset,
				who,
				&escrow,
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

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
	}
}
