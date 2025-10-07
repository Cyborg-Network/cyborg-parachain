use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, TypeInfo, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum PaymentMode {
    OnDemand,   // Pay as you go
    Subscription, // Monthly subscription
}

#[derive(Clone, Encode, Decode, TypeInfo, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, DecodeWithMemTracking)]
pub struct PaymentPeriod<BlockNumber> {
    pub start_block: BlockNumber,
    pub end_block: BlockNumber,
}
