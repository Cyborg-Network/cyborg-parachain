#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;


pub trait WeightInfo {
    fn purchase_compute_hours() -> Weight;
    fn consume_compute_hours() -> Weight;
    fn set_price_per_hour() -> Weight;
    fn set_service_provider_account() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_payment::WeightInfo for SubstrateWeight<T> {
    fn purchase_compute_hours() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(T::DbWeight::get().reads(1_u64))
        .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn consume_compute_hours() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(T::DbWeight::get().reads(1_u64))
        .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn set_price_per_hour() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(T::DbWeight::get().reads(1_u64))
        .saturating_add(T::DbWeight::get().writes(2_u64))
    }

    fn set_service_provider_account() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(T::DbWeight::get().reads(1_u64))
        .saturating_add(T::DbWeight::get().writes(2_u64))
    }
}



impl WeightInfo for () {
	fn purchase_compute_hours() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(RocksDbWeight::get().reads(1_u64))
        .saturating_add(RocksDbWeight::get().writes(2_u64))
    }

    fn consume_compute_hours() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(RocksDbWeight::get().reads(1_u64))
        .saturating_add(RocksDbWeight::get().writes(2_u64))
    }

    fn set_price_per_hour() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(RocksDbWeight::get().reads(1_u64))
        .saturating_add(RocksDbWeight::get().writes(2_u64))
    }

    fn set_service_provider_account() -> Weight {
        Weight::from_parts(10_000, 3513)
        .saturating_add(RocksDbWeight::get().reads(1_u64))
        .saturating_add(RocksDbWeight::get().writes(2_u64))
    }

}
