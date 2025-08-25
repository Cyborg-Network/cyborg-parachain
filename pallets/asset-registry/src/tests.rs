use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use xcm::latest::prelude::*;

fn asset_id(id: u128) -> AssetId {
	AssetId(Location::new(0, [GeneralIndex(id)]))
}

#[test]
fn register_asset_works() {
	new_test_ext().execute_with(|| {
		let asset_id_val = asset_id(1984); // USDT asset ID
		let location = Location::new(1, [Parachain(1000), GeneralIndex(1984)]);

		// Only root can register assets
		assert_noop!(
			AssetRegistry::register_asset(
				RuntimeOrigin::signed(1),
				asset_id_val.clone(),
				location.clone(),
				Some(6)
			),
			DispatchError::BadOrigin
		);

		// Root can register assets
		assert_ok!(AssetRegistry::register_asset(
			RuntimeOrigin::root(),
			asset_id_val.clone(),
			location.clone(),
			Some(6)
		));

		// Check storage
		assert!(AssetRegistry::is_asset_registered(asset_id_val.clone()));
		let (stored_location, decimals) =
			AssetRegistry::registered_assets(asset_id_val.clone()).unwrap();
		assert_eq!(stored_location, location);
		assert_eq!(decimals, Some(6));

		// Check event
		System::assert_last_event(RuntimeEvent::AssetRegistry(Event::AssetRegistered {
			asset_id: asset_id_val.clone(),
			location,
		}));
	});
}

#[test]
fn register_duplicate_asset_fails() {
	new_test_ext().execute_with(|| {
		let asset_id_val = asset_id(1984);
		let location = Location::new(1, [Parachain(1000), GeneralIndex(1984)]);

		assert_ok!(AssetRegistry::register_asset(
			RuntimeOrigin::root(),
			asset_id_val.clone(),
			location.clone(),
			Some(6)
		));

		// Cannot register duplicate asset
		assert_noop!(
			AssetRegistry::register_asset(
				RuntimeOrigin::root(),
				asset_id_val.clone(),
				location,
				Some(6)
			),
			Error::<Test>::AssetAlreadyRegistered
		);
	});
}

#[test]
fn transfer_to_asset_hub_works() {
	new_test_ext().execute_with(|| {
		let asset_id_val = asset_id(1984);
		let location = Location::new(1, [Parachain(1000), GeneralIndex(1984)]);
		let beneficiary = [1u8; 32];
		let amount = 1000;

		// Register asset first
		assert_ok!(AssetRegistry::register_asset(
			RuntimeOrigin::root(),
			asset_id_val.clone(),
			location.clone(),
			Some(6)
		));

		// Transfer to Asset Hub
		assert_ok!(AssetRegistry::transfer_to_asset_hub(
			RuntimeOrigin::signed(1),
			asset_id_val.clone(),
			amount,
			beneficiary
		));

		// Check event
		System::assert_last_event(RuntimeEvent::AssetRegistry(Event::AssetTransferInitiated {
			asset_id: asset_id_val.clone(),
			amount,
			destination: Location::new(1, [Parachain(1000)]),
			initiator: 1,
		}));
	});
}

#[test]
fn transfer_unregistered_asset_fails() {
	new_test_ext().execute_with(|| {
		let asset_id_val = asset_id(9999); // Unregistered asset
		let beneficiary = [1u8; 32];
		let amount = 1000;

		assert_noop!(
			AssetRegistry::transfer_to_asset_hub(
				RuntimeOrigin::signed(1),
				asset_id_val.clone(),
				amount,
				beneficiary
			),
			Error::<Test>::AssetNotFound
		);
	});
}

#[test]
fn get_all_assets_works() {
	new_test_ext().execute_with(|| {
		let asset1_id = asset_id(1984);
		let asset1_location = Location::new(1, [Parachain(1000), GeneralIndex(1984)]);

		let asset2_id = asset_id(1234);
		let asset2_location = Location::new(1, [Parachain(1000), GeneralIndex(1234)]);

		// Register assets
		assert_ok!(AssetRegistry::register_asset(
			RuntimeOrigin::root(),
			asset1_id.clone(),
			asset1_location.clone(),
			Some(6)
		));

		assert_ok!(AssetRegistry::register_asset(
			RuntimeOrigin::root(),
			asset2_id.clone(),
			asset2_location.clone(),
			Some(12)
		));

		// Get all assets
		let assets = AssetRegistry::get_all_assets();
		assert_eq!(assets.len(), 2);

		assert!(assets
			.iter()
			.any(|(id, loc, dec)| *id == asset1_id && *loc == asset1_location && *dec == Some(6)));

		assert!(assets
			.iter()
			.any(|(id, loc, dec)| *id == asset2_id && *loc == asset2_location && *dec == Some(12)));
	});
}
