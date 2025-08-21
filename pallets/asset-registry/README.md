# Pallet Asset Registry

A Substrate pallet for managing cross-chain asset registrations and XCM transfers to Polkadot's Asset Hub.

## Overview

This pallet provides centralized management of cross-chain assets, enabling seamless integration with Polkadot's Asset Hub through XCM (Cross-Consensus Messaging). It allows governance-controlled asset registration and user-initiated asset transfers.

## Features

- **Governance-controlled Asset Registration**: Root-only registration of cross-chain assets
- **XCM Transfer Execution**: Built-in XCM message construction for asset transfers
- **Asset Metadata Storage**: Central repository for asset locations and decimal information
- **Event Emission**: Comprehensive event tracking for all operations
- **Security First**: Proper error handling and access control

## Storage

### `RegisteredAssets`
Maps asset IDs to their XCM location and decimal information:
```rust
StorageMap<AssetId, (Location, Option<u128>)>
```

### `Extrinsics`
register_asset
Register a new cross-chain asset (root-only):

```rust
fn register_asset(
    origin: OriginFor<T>,
    asset_id: AssetId,
    location: Location,
    decimals: Option<u128>,
) -> DispatchResult
```

transfer_to_asset_hub
Transfer assets to Asset Hub:

```rust
fn transfer_to_asset_hub(
    origin: OriginFor<T>,
    asset_id: AssetId,
    amount: u128,
    beneficiary: [u8; 32],
) -> DispatchResult
```


# Events
- AssetRegistered: Emitted when a new asset is registered

- AssetTransferInitiated: Emitted when an asset transfer is initiated

# Errors
- AssetAlreadyRegistered: Attempt to register an existing asset

- AssetNotFound: Attempt to transfer an unregistered asset

- XcmExecutionFailed: XCM message execution failed