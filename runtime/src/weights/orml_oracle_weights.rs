
//! Autogenerated weights for `orml_oracle`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 39.0.0
//! DATE: 2024-09-24, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `SGOWMBP3`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// ./target/release/cyborg-node
// benchmark
// pallet
// --runtime=./target/release/wbuild/cyborg-runtime/cyborg_runtime.wasm
// --genesis-builder=runtime
// --pallet=orml_oracle
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=.maintain/frame-weight-template.hbs
// --output=./runtime/src/weights/orml_oracle_weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `orml_oracle`.
pub trait WeightInfo {
	fn feed_values(x: u32, ) -> Weight;
	fn on_finalize() -> Weight;
}

/// Weights for `orml_oracle` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `OracleMembership::Members` (r:1 w:0)
	/// Proof: `OracleMembership::Members` (`max_values`: Some(1), `max_size`: Some(513), added: 1008, mode: `MaxEncodedLen`)
	/// Storage: `Oracle::HasDispatched` (r:1 w:1)
	/// Proof: `Oracle::HasDispatched` (`max_values`: Some(1), `max_size`: Some(257), added: 752, mode: `MaxEncodedLen`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Oracle::RawValues` (r:99 w:99)
	/// Proof: `Oracle::RawValues` (`max_values`: None, `max_size`: Some(98), added: 2573, mode: `MaxEncodedLen`)
	/// Storage: `Oracle::Values` (r:99 w:0)
	/// Proof: `Oracle::Values` (`max_values`: None, `max_size`: Some(58), added: 2533, mode: `MaxEncodedLen`)
	/// Storage: `EdgeConnect::WorkerClusters` (r:99 w:0)
	/// Proof: `EdgeConnect::WorkerClusters` (`max_values`: None, `max_size`: Some(227), added: 2702, mode: `MaxEncodedLen`)
	/// The range of component `x` is `[0, 99]`.
	fn feed_values(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `39`
		//  Estimated: `1998 + x * (2702 ±0)`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 1998)
			// Standard Error: 97_058
			.saturating_add(Weight::from_parts(19_990_326, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(x.into())))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x.into())))
			.saturating_add(Weight::from_parts(0, 2702).saturating_mul(x.into()))
	}
	/// Storage: `Oracle::HasDispatched` (r:0 w:1)
	/// Proof: `Oracle::HasDispatched` (`max_values`: Some(1), `max_size`: Some(257), added: 752, mode: `MaxEncodedLen`)
	fn on_finalize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_000_000 picoseconds.
		Weight::from_parts(1_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `OracleMembership::Members` (r:1 w:0)
	/// Proof: `OracleMembership::Members` (`max_values`: Some(1), `max_size`: Some(513), added: 1008, mode: `MaxEncodedLen`)
	/// Storage: `Oracle::HasDispatched` (r:1 w:1)
	/// Proof: `Oracle::HasDispatched` (`max_values`: Some(1), `max_size`: Some(257), added: 752, mode: `MaxEncodedLen`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `Oracle::RawValues` (r:99 w:99)
	/// Proof: `Oracle::RawValues` (`max_values`: None, `max_size`: Some(98), added: 2573, mode: `MaxEncodedLen`)
	/// Storage: `Oracle::Values` (r:99 w:0)
	/// Proof: `Oracle::Values` (`max_values`: None, `max_size`: Some(58), added: 2533, mode: `MaxEncodedLen`)
	/// Storage: `EdgeConnect::WorkerClusters` (r:99 w:0)
	/// Proof: `EdgeConnect::WorkerClusters` (`max_values`: None, `max_size`: Some(227), added: 2702, mode: `MaxEncodedLen`)
	/// The range of component `x` is `[0, 99]`.
	fn feed_values(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `39`
		//  Estimated: `1998 + x * (2702 ±0)`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 1998)
			// Standard Error: 97_058
			.saturating_add(Weight::from_parts(19_990_326, 0).saturating_mul(x.into()))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().reads((3_u64).saturating_mul(x.into())))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(x.into())))
			.saturating_add(Weight::from_parts(0, 2702).saturating_mul(x.into()))
	}
	/// Storage: `Oracle::HasDispatched` (r:0 w:1)
	/// Proof: `Oracle::HasDispatched` (`max_values`: Some(1), `max_size`: Some(257), added: 752, mode: `MaxEncodedLen`)
	fn on_finalize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_000_000 picoseconds.
		Weight::from_parts(1_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}