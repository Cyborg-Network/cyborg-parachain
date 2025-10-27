use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

// Struct to hold reward rates per resource type.
#[derive(
	Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct RewardRates<Balance> {
	pub cpu: Balance,
	pub ram: Balance,
	pub storage: Balance,
}

// Supported payment assets
#[derive(
	Encode,
	Decode,
	Clone,
	PartialEq,
	Eq,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
	Copy,
	DecodeWithMemTracking,
)]
pub enum PaymentAsset {
	Native,
	USDT,
	USDC,
	BORG,
	DOT,
}

impl Default for PaymentAsset {
	fn default() -> Self {
		Self::Native
	}
}

// Asset IDs configuration - these should match what's registered on AssetHub
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct AssetConfig {
	pub asset_id: u32,
	pub decimals: u8,
	pub min_amount: u128,
}
