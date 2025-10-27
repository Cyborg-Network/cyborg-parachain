#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::DispatchResult;
	use pallet_assets as assets;

	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	impl<T: Config> Pallet<T> {
		/// Transfer assets from one account to another
		pub fn transfer(
			from: T::AccountId,
			to: T::AccountId,
			asset_id: u32,
			amount: u128,
		) -> DispatchResult
		where
			T::AssetId: From<u32>,
			T::Balance: From<u128>,
		{
			// Convert parameters to the types expected by pallet_assets
			let asset_id_converted: T::AssetId = asset_id.into();
			let asset_amount: T::Balance = amount.into();

			// Convert the destination address to the proper lookup type
			let dest = <T::Lookup as sp_runtime::traits::StaticLookup>::unlookup(to);

			assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(from).into(),
				asset_id_converted.into(),
				dest,
				asset_amount,
			)?;

			Ok(())
		}

		/// Get asset balance
		pub fn balance(asset_id: u32, who: &T::AccountId) -> u128
		where
			T::AssetId: From<u32>,
			T::Balance: Into<u128>,
		{
			let asset_id_converted: T::AssetId = asset_id.into();
			let balance: T::Balance = assets::Pallet::<T>::balance(asset_id_converted, who);
			balance.into()
		}

		/// Check if an account has sufficient asset balance
		pub fn has_sufficient_balance(who: &T::AccountId, asset_id: u32, amount: u128) -> bool
		where
			T::AssetId: From<u32>,
			T::Balance: Into<u128>,
		{
			Self::balance(asset_id, who) >= amount
		}
	}
}

