
//! Autogenerated weights for `cumulus_pallet_xcmp_queue`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 39.0.0
//! DATE: 2024-09-25, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `SGOWMBP3`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/cyborg-node
// benchmark
// pallet
// --chain=dev
// --pallet=cumulus_pallet_xcmp_queue
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=.maintain/frame-weight-template.hbs
// --output=./runtime/src/weights/cumulus_pallet_xcmp_queue.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `cumulus_pallet_xcmp_queue`.
pub trait WeightInfo {
	fn set_config_with_u32() -> Weight;
	fn enqueue_xcmp_message() -> Weight;
	fn suspend_channel() -> Weight;
	fn resume_channel() -> Weight;
	fn take_first_concatenated_xcm() -> Weight;
	fn on_idle_good_msg() -> Weight;
	fn on_idle_large_msg() -> Weight;
}

/// Weights for `cumulus_pallet_xcmp_queue` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> cumulus_pallet_xcmp_queue::WeightInfo for SubstrateWeight<T> {
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:1)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	fn set_config_with_u32() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `1497`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(4_000_000, 1497)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:0)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:1)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::ServiceHead` (r:1 w:1)
	/// Proof: `MessageQueue::ServiceHead` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::InboundXcmpSuspended` (r:1 w:0)
	/// Proof: `XcmpQueue::InboundXcmpSuspended` (`max_values`: Some(1), `max_size`: Some(4002), added: 4497, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::Pages` (r:0 w:1)
	/// Proof: `MessageQueue::Pages` (`max_values`: None, `max_size`: Some(105521), added: 107996, mode: `MaxEncodedLen`)
	fn enqueue_xcmp_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `82`
		//  Estimated: `5487`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(11_000_000, 5487)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `XcmpQueue::OutboundXcmpStatus` (r:1 w:1)
	/// Proof: `XcmpQueue::OutboundXcmpStatus` (`max_values`: Some(1), `max_size`: Some(1282), added: 1777, mode: `MaxEncodedLen`)
	fn suspend_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `2767`
		// Minimum execution time: 2_000_000 picoseconds.
		Weight::from_parts(2_000_000, 2767)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `XcmpQueue::OutboundXcmpStatus` (r:1 w:1)
	/// Proof: `XcmpQueue::OutboundXcmpStatus` (`max_values`: Some(1), `max_size`: Some(1282), added: 1777, mode: `MaxEncodedLen`)
	fn resume_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `111`
		//  Estimated: `2767`
		// Minimum execution time: 2_000_000 picoseconds.
		Weight::from_parts(3_000_000, 2767)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn take_first_concatenated_xcm() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(9_000_000, 0)
	}
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:1)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::ServiceHead` (r:1 w:1)
	/// Proof: `MessageQueue::ServiceHead` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:0)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::InboundXcmpSuspended` (r:1 w:0)
	/// Proof: `XcmpQueue::InboundXcmpSuspended` (`max_values`: Some(1), `max_size`: Some(4002), added: 4497, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::Pages` (r:0 w:1)
	/// Proof: `MessageQueue::Pages` (`max_values`: None, `max_size`: Some(105521), added: 107996, mode: `MaxEncodedLen`)
	fn on_idle_good_msg() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `105647`
		//  Estimated: `109112`
		// Minimum execution time: 58_000_000 picoseconds.
		Weight::from_parts(59_000_000, 109112)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:1)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::ServiceHead` (r:1 w:1)
	/// Proof: `MessageQueue::ServiceHead` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:0)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::InboundXcmpSuspended` (r:1 w:0)
	/// Proof: `XcmpQueue::InboundXcmpSuspended` (`max_values`: Some(1), `max_size`: Some(4002), added: 4497, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::Pages` (r:0 w:1)
	/// Proof: `MessageQueue::Pages` (`max_values`: None, `max_size`: Some(105521), added: 107996, mode: `MaxEncodedLen`)
	fn on_idle_large_msg() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `65716`
		//  Estimated: `69181`
		// Minimum execution time: 43_000_000 picoseconds.
		Weight::from_parts(44_000_000, 69181)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:1)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	fn set_config_with_u32() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `1497`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(4_000_000, 1497)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:0)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:1)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::ServiceHead` (r:1 w:1)
	/// Proof: `MessageQueue::ServiceHead` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::InboundXcmpSuspended` (r:1 w:0)
	/// Proof: `XcmpQueue::InboundXcmpSuspended` (`max_values`: Some(1), `max_size`: Some(4002), added: 4497, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::Pages` (r:0 w:1)
	/// Proof: `MessageQueue::Pages` (`max_values`: None, `max_size`: Some(105521), added: 107996, mode: `MaxEncodedLen`)
	fn enqueue_xcmp_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `82`
		//  Estimated: `5487`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(11_000_000, 5487)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `XcmpQueue::OutboundXcmpStatus` (r:1 w:1)
	/// Proof: `XcmpQueue::OutboundXcmpStatus` (`max_values`: Some(1), `max_size`: Some(1282), added: 1777, mode: `MaxEncodedLen`)
	fn suspend_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `2767`
		// Minimum execution time: 2_000_000 picoseconds.
		Weight::from_parts(2_000_000, 2767)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `XcmpQueue::OutboundXcmpStatus` (r:1 w:1)
	/// Proof: `XcmpQueue::OutboundXcmpStatus` (`max_values`: Some(1), `max_size`: Some(1282), added: 1777, mode: `MaxEncodedLen`)
	fn resume_channel() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `111`
		//  Estimated: `2767`
		// Minimum execution time: 2_000_000 picoseconds.
		Weight::from_parts(3_000_000, 2767)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn take_first_concatenated_xcm() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(9_000_000, 0)
	}
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:1)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::ServiceHead` (r:1 w:1)
	/// Proof: `MessageQueue::ServiceHead` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:0)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::InboundXcmpSuspended` (r:1 w:0)
	/// Proof: `XcmpQueue::InboundXcmpSuspended` (`max_values`: Some(1), `max_size`: Some(4002), added: 4497, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::Pages` (r:0 w:1)
	/// Proof: `MessageQueue::Pages` (`max_values`: None, `max_size`: Some(105521), added: 107996, mode: `MaxEncodedLen`)
	fn on_idle_good_msg() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `105647`
		//  Estimated: `109112`
		// Minimum execution time: 58_000_000 picoseconds.
		Weight::from_parts(59_000_000, 109112)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6b345d8e88afa015075c945637c07e8f20` (r:1 w:1)
	/// Storage: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Proof: UNKNOWN KEY `0x7b3237373ffdfeb1cab4222e3b520d6bedc49980ba3aa32b0a189290fd036649` (r:1 w:1)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:1)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::ServiceHead` (r:1 w:1)
	/// Proof: `MessageQueue::ServiceHead` (`max_values`: Some(1), `max_size`: Some(5), added: 500, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::QueueConfig` (r:1 w:0)
	/// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: Some(12), added: 507, mode: `MaxEncodedLen`)
	/// Storage: `XcmpQueue::InboundXcmpSuspended` (r:1 w:0)
	/// Proof: `XcmpQueue::InboundXcmpSuspended` (`max_values`: Some(1), `max_size`: Some(4002), added: 4497, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::Pages` (r:0 w:1)
	/// Proof: `MessageQueue::Pages` (`max_values`: None, `max_size`: Some(105521), added: 107996, mode: `MaxEncodedLen`)
	fn on_idle_large_msg() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `65716`
		//  Estimated: `69181`
		// Minimum execution time: 43_000_000 picoseconds.
		Weight::from_parts(44_000_000, 69181)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
}
