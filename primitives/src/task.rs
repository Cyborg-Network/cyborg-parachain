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

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum TaskSubmissionData {
	NeuroZK(NeuroZkTaskSubmissionDetails),       // A Zero-Knowledge Proof Generation task.
	OpenInference(OpenInferenceTask), // An AI Inference Task (normal).
	FlashInfer(FlashInferTask),
}

/// Kinds of overall tasks at a logical level (business logic: inference vs zk proof).
#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum TaskKind<BlockNumber> {
	NeuroZK(NzkData<BlockNumber>),       // A Zero-Knowledge Proof Generation task.
	OpenInference(OpenInferenceTask), // An AI Inference Task (normal).
	FlashInferInfer(FlashInferTask)
}

impl<BlockNumber> TaskKind<BlockNumber> {
	pub fn from_submission(submitted: TaskSubmissionData) -> Self {
		match submitted {
			TaskSubmissionData::NeuroZK(details) => {
				TaskKind::NeuroZK(NzkData { 
					location: details.location,
					zk_input: details.zk_input, 
					zk_settings: details.zk_settings, 
					zk_verifying_key: details.zk_verifying_key, 
					zk_proof: None, 
					last_proof_accepted: None,
				})
			},
			TaskSubmissionData::OpenInference(task) => TaskKind::OpenInference(task),
			TaskSubmissionData::FlashInfer(task) => TaskKind::FlashInferInfer(task),
		}
	}
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum OpenInferenceTask {
	Onnx(OnnxTask),
	//Cess(CessTask),
	//Azure(AzureTask),
	//Huggingface(HuggingfaceTask),
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum FlashInferTask {
	Huggingface(HuggingfaceTask),
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub struct OnnxTask {
	pub storage_location_identifier: BoundedVec<u8, ConstU32<500>>,
	pub triton_config: Option<BoundedVec<u8, ConstU32<500>>>,
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub struct CessTask {
	pub storage_location_identifier: BoundedVec<u8, ConstU32<500>>,
	pub dh_pub_key: BoundedVec<u8, ConstU32<500>>,
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub struct AzureTask {
	pub storage_location_identifier: BoundedVec<u8, ConstU32<500>>,
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub struct HuggingfaceTask {
	pub hf_identifier: BoundedVec<u8, ConstU32<500>>,
}

///Detailed information about a specific task.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TaskInfo<AccountId, BlockNumber> {
	pub task_owner: AccountId,                   // Who scheduled the task.
	pub create_block: BlockNumber,               // Block when created.
	pub time_elapsed: Option<BlockNumber>,       // Time consumed.
	pub average_cpu_percentage_use: Option<u8>,  // CPU usage.
	pub task_kind: TaskKind<BlockNumber>,                     // New: Logical kind (NeuroZK or OpenInference).
	pub result: Option<BoundedVec<u8, ConstU32<500>>>, // Final result (optional).
	pub compute_hours_deposit: Option<u32>,      // Deposit paid upfront.
	pub consume_compute_hours: Option<u32>,      // How much was actually consumed.
	pub task_status: TaskStatusType,             // Current lifecycle status.
}

pub type ZkInput = BoundedVec<u8, ConstU32<5000>>;
pub type ZkSettings = BoundedVec<u8, ConstU32<5000>>;
pub type ZkVerifyingKey = BoundedVec<u8, ConstU32<500000>>;
pub type ZkProof = BoundedVec<u8, ConstU32<50000>>;

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, DecodeWithMemTracking)]
pub struct NeuroZkTaskSubmissionDetails {
	pub location: AzureTask,
	pub zk_input: ZkInput,
	pub zk_settings: ZkSettings,
	pub zk_verifying_key: ZkVerifyingKey,
}

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, DecodeWithMemTracking)]
pub struct NzkData<BlockNumber> {
	pub location: AzureTask,
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