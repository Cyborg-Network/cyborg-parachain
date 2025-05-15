use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;

// Struct to hold reward rates per resource type.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardRates<Balance> {
	pub cpu: Balance,
	pub ram: Balance,
	pub storage: Balance,
}
