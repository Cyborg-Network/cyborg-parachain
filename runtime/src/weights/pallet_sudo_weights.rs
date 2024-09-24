
//! Autogenerated weights for `pallet_sudo`
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
// --pallet=pallet_sudo
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=.maintain/frame-weight-template.hbs
// --output=./runtime/src/weights/pallet_sudo_weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_sudo`.
pub trait WeightInfo {
	fn set_key() -> Weight;
	fn sudo() -> Weight;
	fn sudo_as() -> Weight;
	fn remove_key() -> Weight;
}

/// Weights for `pallet_sudo` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Sudo::Key` (r:1 w:1)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn set_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(6_000_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn sudo() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn sudo_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:1)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn remove_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(5_000_000, 1517)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Sudo::Key` (r:1 w:1)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn set_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(6_000_000, 1517)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn sudo() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 1517)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:0)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn sudo_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(7_000_000, 1517)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: `Sudo::Key` (r:1 w:1)
	/// Proof: `Sudo::Key` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	fn remove_key() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `37`
		//  Estimated: `1517`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(5_000_000, 1517)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
