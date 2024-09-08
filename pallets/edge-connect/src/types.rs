use scale_info::{ TypeInfo };
use frame_support::{sp_runtime::RuntimeDebug, BoundedVec, pallet_prelude::ConstU32 };
use codec::{ Encode, Decode, MaxEncodedLen };

pub type WorkerId = u64;

pub type Domain = BoundedVec<u8, ConstU32<128>>;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum WorkerStatusType {
	Active,
	Busy,
	Inactive,
}

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen )]
pub struct WorkerAPI {
	pub domain: Domain,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Worker<AccountId, BlockNumber> {
	pub id: WorkerId,
	pub owner: AccountId,
	pub start_block: BlockNumber,
	pub status: WorkerStatusType,
	pub api: WorkerAPI,
}