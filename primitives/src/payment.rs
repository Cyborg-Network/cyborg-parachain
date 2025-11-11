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
pub struct PaymentPeriod<BlockNumber> {
    pub start_block: BlockNumber,
    pub end_block: BlockNumber,
    pub purpose: PaymentPurpose,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq, TypeInfo, MaxEncodedLen, DecodeWithMemTracking)]
pub enum PaymentPurpose {
    TaskExecution(TaskId), // Payment is for specific task execution
    General,               // General subscription (for future use)
}
