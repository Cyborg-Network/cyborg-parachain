use codec::{Decode, Encode, MaxEncodedLen, DecodeWithMemTracking};
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
#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
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
	pub nzk_data: Option<NzkData<BlockNumber>>,  // Optional: NZK data if task is NZK.
	pub time_elapsed: Option<BlockNumber>,       // Time consumed.
	pub average_cpu_percentage_use: Option<u8>,  // CPU usage.
	pub task_kind: TaskKind,                     // New: Logical kind (NeuroZK or OpenInference).
	pub result: Option<BoundedVec<u8, ConstU32<500>>>, // Final result (optional).
	pub compute_hours_deposit: Option<u32>,      // Deposit paid upfront.
	pub consume_compute_hours: Option<u32>,      // How much was actually consumed.
	pub task_status: TaskStatusType,             // Current lifecycle status.
}

pub type ZkInput = BoundedVec<u8, ConstU32<5000>>;
pub type ZkSettings = BoundedVec<u8, ConstU32<5000>>;
pub type ZkVerifyingKey = BoundedVec<u8, ConstU32<500000>>;
pub type ZkProof = BoundedVec<u8, ConstU32<50000>>;

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen, PartialEq, Debug, DecodeWithMemTracking)]
pub struct NeuroZkTaskSubmissionDetails {
	pub zk_input: ZkInput,
	pub zk_settings: ZkSettings,
	pub zk_verifying_key: ZkVerifyingKey,
}

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct NzkData<BlockNumber> {
	pub zk_input: ZkInput,
	pub zk_settings: ZkSettings,
	pub zk_verifying_key: ZkVerifyingKey,
	pub zk_proof: Option<ZkProof>,
	pub last_proof_accepted: Option<(bool, BlockNumber)>,
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

pub trait NzkTaskInfoHandler<AccountId, TaskId, BlockNumber> {
	fn get_nzk_task(task_id: TaskId) -> Option<TaskInfo<AccountId, BlockNumber>>;
	fn update_nzk_task(task_id: TaskId, task: TaskInfo<AccountId, BlockNumber>);
}
