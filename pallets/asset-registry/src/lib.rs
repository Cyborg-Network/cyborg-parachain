#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use xcm::latest::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type XcmExecutor: xcm_executor::Config;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
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
        AssetRegistered { asset_id: AssetId, location: Location },
        AssetTransferInitiated { asset_id: AssetId, amount: u128, destination: Location },
    }

    #[pallet::error]
    pub enum Error<T> {
        AssetAlreadyRegistered,
        AssetNotFound,
        XcmExecutionFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_asset(
            origin: OriginFor<T>,
            asset_id: AssetId,
            location: Location,
            decimals: Option<u128>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                !RegisteredAssets::<T>::contains_key(asset_id),
                Error::<T>::AssetAlreadyRegistered
            );

            RegisteredAssets::<T>::insert(asset_id, (location, decimals));
            Self::deposit_event(Event::AssetRegistered { asset_id, location });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn transfer_to_asset_hub(
            origin: OriginFor<T>,
            asset_id: AssetId,
            amount: u128,
            beneficiary: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let (location, _) = RegisteredAssets::<T>::get(asset_id)
                .ok_or(Error::<T>::AssetNotFound)?;

            // Create XCM message to transfer assets to Asset Hub
            let message = Xcm(vec![
                WithdrawAsset(Asset {
                    id: AssetId(location.clone()),
                    fun: Fungible(amount),
                }.into()),
                InitiateReserveWithdraw {
                    assets: Wild(All),
                    reserve: Location::new(1, [Parachain(1000)]), // Asset Hub
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

            // Execute XCM message
            let _ = xcm_executor::execute_xcm(
                Location::new(1, [Parachain(1000)]),
                message,
                Weight::from_parts(1_000_000_000, 64 * 1024),
            ).map_err(|_| Error::<T>::XcmExecutionFailed)?;

            Self::deposit_event(Event::AssetTransferInitiated {
                asset_id,
                amount,
                destination: Location::new(1, [Parachain(1000)]),
            });

            Ok(())
        }
    }
}