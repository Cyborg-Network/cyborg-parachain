use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Get},
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{DispatchResult, MultiAddress};
use sp_std::vec::Vec;
use xcm::opaque::v3::MultiLocation;

use crate::{AccountId, Balance, Runtime, TreasuryAccount};

/// Asset metadata structure
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AssetMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
		type TreasuryAccount: Get<Self::AccountId>;
	}

	/// Mapping from local asset ID to XCM multi-location
	#[pallet::storage]
	#[pallet::getter(fn asset_locations)]
	pub type AssetLocations<T> = StorageMap<_, Blake2_128Concat, u32, MultiLocation, OptionQuery>;

	/// Mapping from XCM multi-location to local asset ID
	#[pallet::storage]
	#[pallet::getter(fn location_to_asset)]
	pub type LocationToAsset<T> = StorageMap<_, Blake2_128Concat, MultiLocation, u32, OptionQuery>;

	/// Asset metadata storage
	#[pallet::storage]
	#[pallet::getter(fn asset_metadatas)]
	pub type AssetMetadatas<T> = StorageMap<_, Blake2_128Concat, u32, AssetMetadata, OptionQuery>;

	/// Next available asset ID
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetRegistered {
			asset_id: u32,
			location: MultiLocation,
		},
		AssetMinted {
			asset_id: u32,
			beneficiary: T::AccountId,
			amount: Balance,
		},
		AssetBurned {
			asset_id: u32,
			account: T::AccountId,
			amount: Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		AssetAlreadyRegistered,
		AssetNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a foreign asset with a local asset ID
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 3))]
		pub fn register_asset(
			origin: OriginFor<T>,
			asset_location: MultiLocation,
			metadata: AssetMetadata,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Ensure the asset location is not already registered
			if LocationToAsset::<T>::contains_key(&asset_location) {
				return Err(Error::<T>::AssetAlreadyRegistered.into());
			}

			// Get the next available asset ID
			let asset_id = NextAssetId::<T>::get();
			NextAssetId::<T>::put(asset_id + 1);

			// Store the mappings
			AssetLocations::<T>::insert(asset_id, asset_location.clone());
			LocationToAsset::<T>::insert(asset_location.clone(), asset_id);
			AssetMetadatas::<T>::insert(asset_id, metadata.clone());

			// Create the asset in the assets pallet
			pallet_assets::Pallet::<Runtime>::create(
				frame_system::RawOrigin::Root.into(),
				asset_id.into(),
				sp_runtime::MultiAddress::Id(T::TreasuryAccount::get()),
				Balance::from(1u32), // minimal balance
			)?;

			// Set asset metadata
			pallet_assets::Pallet::<Runtime>::set_metadata(
				frame_system::RawOrigin::Root.into(),
				asset_id.into(),
				metadata.name.clone(),
				metadata.symbol.clone(),
				metadata.decimals,
			)?;

			Self::deposit_event(Event::AssetRegistered {
				asset_id,
				location: asset_location,
			});
			Ok(())
		}

		/// Mint some amount of a foreign asset to an account
		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn mint_asset(
			origin: OriginFor<T>,
			asset_id: u32,
			beneficiary: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Ensure asset exists
			ensure!(
				AssetLocations::<T>::contains_key(asset_id),
				Error::<T>::AssetNotFound
			);

			pallet_assets::Pallet::<Runtime>::mint(
				frame_system::RawOrigin::Root.into(),
				asset_id.into(),
				MultiAddress::Id(beneficiary.clone()),
				amount,
			)?;

			Self::deposit_event(Event::AssetMinted {
				asset_id,
				beneficiary,
				amount,
			});
			Ok(())
		}

		/// Burn some amount of a foreign asset from an account
		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn burn_asset(
			origin: OriginFor<T>,
			asset_id: u32,
			account: T::AccountId,
			amount: Balance,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Ensure asset exists
			ensure!(
				AssetLocations::<T>::contains_key(asset_id),
				Error::<T>::AssetNotFound
			);

			pallet_assets::Pallet::<Runtime>::burn(
				frame_system::RawOrigin::Root.into(),
				asset_id.into(),
				MultiAddress::Id(account.clone()),
				amount,
			)?;

			Self::deposit_event(Event::AssetBurned {
				asset_id,
				account,
				amount,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get XCM location for a registered asset
		pub fn get_asset_location(asset_id: u32) -> Option<MultiLocation> {
			AssetLocations::<T>::get(asset_id)
		}

		/// Get local asset ID for an XCM location
		pub fn get_local_asset_id(asset_location: MultiLocation) -> Option<u32> {
			LocationToAsset::<T>::get(asset_location)
		}

		/// Get metadata for a registered asset
		pub fn get_asset_metadata(asset_id: u32) -> Option<AssetMetadata> {
			AssetMetadatas::<T>::get(asset_id)
		}

		/// Check if an asset is registered
		pub fn is_asset_registered(asset_id: u32) -> bool {
			AssetLocations::<T>::contains_key(asset_id)
		}

		/// Check if a location is registered
		pub fn is_location_registered(asset_location: MultiLocation) -> bool {
			LocationToAsset::<T>::contains_key(asset_location)
		}

		// Predefined asset locations for common Asset Hub assets
		/// Get the MultiLocation for USDT on Asset Hub
		pub fn usdt_location() -> MultiLocation {
			MultiLocation::new(
				1,
				[
					xcm::v3::Junction::Parachain(1000),    // Asset Hub parachain ID
					xcm::v3::Junction::GeneralIndex(1984), // USDT asset index
				],
			)
		}

		/// Get the MultiLocation for USDC on Asset Hub
		pub fn usdc_location() -> MultiLocation {
			MultiLocation::new(
				1,
				[
					xcm::v3::Junction::Parachain(1000),    // Asset Hub parachain ID
					xcm::v3::Junction::GeneralIndex(1337), // USDC asset index
				],
			)
		}

		/// Get the MultiLocation for DOT on Relay Chain
		pub fn dot_location() -> MultiLocation {
			MultiLocation::parent()
		}

		/// Get the MultiLocation for Asset Hub's native token
		pub fn asset_hub_native_location() -> MultiLocation {
			MultiLocation::new(1, [xcm::v3::Junction::Parachain(1000)])
		}
	}
}

// Default metadata for common assets
impl AssetMetadata {
	pub fn usdt() -> Self {
		Self {
			name: b"Tether USD".to_vec(),
			symbol: b"USDT".to_vec(),
			decimals: 6,
		}
	}

	pub fn usdc() -> Self {
		Self {
			name: b"USD Coin".to_vec(),
			symbol: b"USDC".to_vec(),
			decimals: 6,
		}
	}

	pub fn dot() -> Self {
		Self {
			name: b"Polkadot".to_vec(),
			symbol: b"DOT".to_vec(),
			decimals: 10,
		}
	}

	pub fn asset_hub_native() -> Self {
		Self {
			name: b"Asset Hub Native".to_vec(),
			symbol: b"AHN".to_vec(),
			decimals: 12,
		}
	}
}
