use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

pub type WorkerId = u64;

pub type Domain = BoundedVec<u8, ConstU32<128>>;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum WorkerStatusType {
	Active,
	Busy,
	Inactive,
}

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Ip {
	pub ipv4: Option<u8>,
	pub ipv6: Option<u8>,
	pub port: u32,
}

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct WorkerAPI {
	pub ip: Option<Ip>,
	pub domain: Option<Domain>,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Worker<AccountId, BlockNumber> {
	pub id: WorkerId,
	pub owner: AccountId,
	pub start_block: BlockNumber,
	pub status: WorkerStatusType,
	pub api: WorkerAPI,
}
