use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

pub type TaskId = u64;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum TaskStatusType {
	/// Task has been assigned to a worker, but miner hasn't confirmed reception yet.
	Assigned,

	/// Miner has confirmed reception, actively running task.
	Running,

	/// Task was stopped forcibly (admin action or error).
	Stopped,

	/// Miner reset hardware after stopping task.
	Vacated,
}

/// Kinds of overall tasks at a logical level (business logic: inference vs zk proof).
#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum TaskKind {
	NeuroZK,       // A Zero-Knowledge Proof Generation task.
	OpenInference, // An AI Inference Task (normal).
}

///Detailed information about a specific task.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TaskInfo<AccountId, BlockNumber> {
	pub task_owner: AccountId,                   // Who scheduled the task.
	pub create_block: BlockNumber,               // Block when created.
	pub metadata: BoundedVec<u8, ConstU32<500>>, // Could be model link, executable link, etc.
	pub zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>, // Optional: ZK files if needed.
	pub time_elapsed: Option<BlockNumber>,       // Time consumed.
	pub average_cpu_percentage_use: Option<u8>,  // CPU usage.
	pub task_kind: TaskKind,                     // New: Logical kind (NeuroZK or OpenInference).
	pub result: Option<BoundedVec<u8, ConstU32<500>>>, // Final result (optional).
	pub compute_hours_deposit: Option<u32>,      // Deposit paid upfront.
	pub consume_compute_hours: Option<u32>,      // How much was actually consumed.
	pub task_status: TaskStatusType,             // Current lifecycle status.
}

// #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
// pub struct VerificationHashes<AccountId> {
// 	pub account: AccountId,
// 	pub completed_hash: Option<H256>,
// }

// #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
// pub struct Verifications<AccountId> {
// 	pub executor: VerificationHashes<AccountId>,
// 	pub verifier: Option<VerificationHashes<AccountId>>,
// 	pub resolver: Option<VerificationHashes<AccountId>>,
// }
