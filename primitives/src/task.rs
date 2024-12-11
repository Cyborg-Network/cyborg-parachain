use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::{prelude::vec::Vec, TypeInfo};
use sp_core::hash::H256;

pub type TaskId = u64;

/// A vector that contains the IDs of accounts that own a worker that have already been part of the
/// execution / verification / resolution cycle and thus have beeen disqualified from further
/// participation to avoid incorrect or manipulated results.
pub type ForbiddenOwners<AccountId> = Vec<Option<AccountId>>;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum TaskStatusType {
	Assigned,
	PendingValidation,
	Completed,
	Expired,
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum TaskType {
	Docker,
	Executable,
	ZK,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TaskInfo<AccountId, BlockNumber> {
	pub task_owner: AccountId,
	pub create_block: BlockNumber,
	// This is vaguely named as this could be a public docker image or a link to an executable
	pub metadata: BoundedVec<u8, ConstU32<500>>,
	pub zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
	pub time_elapsed: Option<BlockNumber>,
	pub average_cpu_percentage_use: Option<u8>,
	pub task_type: TaskType,
	pub result: Option<BoundedVec<u8, ConstU32<500>>>,
	pub compute_hours_deposit: Option<u32>,
	pub consume_compute_hours: Option<u32>,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerificationHashes<AccountId> {
	pub account: AccountId,
	pub completed_hash: Option<H256>,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Verifications<AccountId> {
	pub executor: VerificationHashes<AccountId>,
	pub verifier: Option<VerificationHashes<AccountId>>,
	pub resolver: Option<VerificationHashes<AccountId>>,
}
