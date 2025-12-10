use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;
use crate::task::TaskId;

// Struct to hold reward rates per resource type.
#[derive(
	Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct RewardRates<Balance> {
	pub cpu: Balance,
	pub ram: Balance,
	pub storage: Balance,
}

#[derive(Clone, Encode, Decode, TypeInfo, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum PaymentMode {
    OnDemand,   // Pay as you go
    Subscription, // Monthly subscription
}

#[derive(Clone, Encode, Decode, TypeInfo, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, DecodeWithMemTracking)]
pub struct PaymentDetails<BlockNumber, Asset, Balance> {
    pub begin: BlockNumber,
    pub expiry: BlockNumber,
    pub asset: Asset,
    pub amount: Balance,
    pub mode: PaymentMode,
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug, Eq, PartialEq, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub enum PaymentPurpose {
    TaskExecution(TaskId), // Payment is for specific task execution
    #[default]
    Undefined,             // Default/unspecified payment purpose
}

/// Trait for getting payment rates based on asset ID	
pub trait PaymentRates<Asset, Balance> {
    /// Get a rate for a specific asset and payment mode	
    fn get_rate(asset_id: Asset, mode: PaymentMode) -> Balance;	
}
