//! Weights for pallet_asset_registry
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};

use core::marker::PhantomData;

pub trait WeightInfo {
    fn register_asset() -> Weight;
    fn transfer_to_asset_hub() -> Weight;
}

/// Weights for pallet_asset_registry using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_asset() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    
    fn transfer_to_asset_hub() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn register_asset() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }
    fn transfer_to_asset_hub() -> Weight {
        Weight::from_parts(35_000_000, 0)
    }
}