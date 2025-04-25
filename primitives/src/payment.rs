use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

// Struct to hold reward rates per resource type.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct RewardRates<Balance> {
		pub cpu: Balance,
		pub ram: Balance,
		pub storage: Balance,
	}
