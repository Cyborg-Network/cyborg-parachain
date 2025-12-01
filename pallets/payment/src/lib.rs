#![cfg_attr(not(feature = "std"), no_std)]

use cyborg_primitives::payment::*;
use frame_support::{sp_runtime::traits::AccountIdConversion, PalletId};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::Saturating,
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
	};

	use super::*;

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

		#[pallet::constant]
		type PalletId: Get<PalletId>;

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

	#[pallet::storage]
	pub type ActivePayments<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
        Blake2_128Concat,
        PaymentPurpose,
		PaymentDetails<BlockNumberFor<T>, BalanceOf<T>>,
	>;

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
        }
	}

	#[pallet::error]
	pub enum Error<T> {
		PaymentAlreadyActive, // Payment is already active (begin and expiry set)
		NoActivePayments,
		InvalidPaymentMode,
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

			Ok(())
		}

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
}
