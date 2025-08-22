#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec;
    use sp_std::vec::Vec;
    use xcm::latest::prelude::*;
    use xcm::v5::Outcome; // Add this import

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// The XCM executor type for executing cross-chain messages
        type XcmExecutor: ExecuteXcm<<Self as frame_system::Config>::RuntimeCall>;
        
        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Storage map for registered cross-chain assets
    #[pallet::storage]
    #[pallet::getter(fn registered_assets)]
    pub type RegisteredAssets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        AssetId,
        (Location, Option<u128>), // (location, decimal places)
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new asset has been registered
        AssetRegistered {
            asset_id: AssetId,
            location: Location,
        },
        /// An asset transfer to Asset Hub has been initiated
        AssetTransferInitiated {
            asset_id: AssetId,
            amount: u128,
            destination: Location,
            initiator: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Asset with this ID is already registered
        AssetAlreadyRegistered,
        /// Asset with this ID is not found
        AssetNotFound,
        /// XCM execution failed
        XcmExecutionFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new cross-chain asset
        ///
        /// Only callable by root (governance)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_asset())]
        pub fn register_asset(
            origin: OriginFor<T>,
            asset_id: AssetId,
            location: Location,
            decimals: Option<u128>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                !RegisteredAssets::<T>::contains_key(&asset_id),
                Error::<T>::AssetAlreadyRegistered
            );

            RegisteredAssets::<T>::insert(&asset_id, (location.clone(), decimals));
            Self::deposit_event(Event::AssetRegistered { 
                asset_id: asset_id.clone(), 
                location: location.clone() 
            });

            Ok(())
        }

        /// Transfer assets to Asset Hub
        ///
        /// Callable by any signed account
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::transfer_to_asset_hub())]
        pub fn transfer_to_asset_hub(
            origin: OriginFor<T>,
            asset_id: AssetId,
            amount: u128,
            beneficiary: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (location, _) = RegisteredAssets::<T>::get(&asset_id)
                .ok_or(Error::<T>::AssetNotFound)?;

            // Create XCM message to transfer assets to Asset Hub
            let message = Xcm(vec![
                WithdrawAsset(vec![Asset {
                    id: AssetId(location.clone()),
                    fun: Fungible(amount),
                }].into()),
                InitiateReserveWithdraw {
                    assets: Wild(All),
                    reserve: Location::new(1, [Parachain(1000)]),
                    xcm: Xcm(vec![
                        DepositAsset {
                            assets: Wild(All),
                            beneficiary: Location::new(
                                0,
                                [AccountId32 { 
                                    network: None, 
                                    id: beneficiary 
                                }]
                            ),
                        },
                    ]),
                },
            ]);

            // Prepare the XCM message for execution
            let prepared = match T::XcmExecutor::prepare(message) {
                Ok(prepared) => prepared,
                Err(_) => return Err(Error::<T>::XcmExecutionFailed.into()),
            };

            // Create a mutable buffer for the XCM hash
            let mut hash_buf = [0u8; 32];
            
            // Execute XCM message using the executor trait - UPDATED METHOD
            let outcome = T::XcmExecutor::execute(
                Location::new(1, [Parachain(1000)]),
                prepared,
                &mut hash_buf,
                Weight::from_parts(1_000_000_000, 64 * 1024),
            );
            
            // Check if the execution was successful
            let is_success = match outcome {
                Outcome::Complete { used: _ } => true,
                Outcome::Incomplete { used: _, error: _ } => false,
                Outcome::Error { error: _ } => false,
            };
            
            ensure!(
                is_success,
                Error::<T>::XcmExecutionFailed
            );

            Self::deposit_event(Event::AssetTransferInitiated {
                asset_id: asset_id.clone(),
                amount,
                destination: Location::new(1, [Parachain(1000)]),
                initiator: who,
            });

            Ok(())
        }
    }

    /// Helper functions for the pallet
    impl<T: Config> Pallet<T> {
        /// Check if an asset is registered
        pub fn is_asset_registered(asset_id: AssetId) -> bool {
            RegisteredAssets::<T>::contains_key(&asset_id)
        }

        /// Get all registered assets
        pub fn get_all_assets() -> Vec<(AssetId, Location, Option<u128>)> {
            RegisteredAssets::<T>::iter()
                .map(|(asset_id, (location, decimals))| (asset_id, location, decimals))
                .collect()
        }
    }
}