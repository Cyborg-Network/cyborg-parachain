use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

pub type TaskId = u64;

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum TaskStatusType {
	/// Task has been assigned to a worker, but miner hasn't confirmed reception yet.
	Assigned,

	/// Miner has confirmed reception, actively running task.
	Running,

	/// Task was stopped forcibly (admin action or error).
	Stopped,

	/// Miner reset hardware after stopping task.
	Vacated,

  	/// The miner failed to run the task
  	Failed,
}

/// Kinds of overall tasks at a logical level (business logic: inference vs zk proof).
#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum TaskKind {
	OpenInference(OpenInferenceTask), // An AI Inference Task (normal).
	FlashInfer(FlashInferTask),
	CyCloud(CyCloudTask),
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum OpenInferenceTask {
	Onnx(OnnxTask),
	//Cess(CessTask),
	//Azure(AzureTask),
	//Huggingface(HuggingfaceTask),
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum CyCloudTask {
  Container(CyCloudContainerTask),
  Native(CyCloudNativeTask),
  Vm(CyCloudVmTask),
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum FlashInferTask {
	Huggingface(HuggingfaceTask),
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct OnnxTask {
	pub storage_location_identifier: BoundedVec<u8, ConstU32<500>>,
	pub triton_config: Option<BoundedVec<u8, ConstU32<500>>>,
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct CessTask {
	pub storage_location_identifier: BoundedVec<u8, ConstU32<500>>,
	pub dh_pub_key: BoundedVec<u8, ConstU32<500>>,
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct AzureTask {
	pub storage_location_identifier: BoundedVec<u8, ConstU32<500>>,
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct HuggingfaceTask {
	pub hf_identifier: BoundedVec<u8, ConstU32<500>>,
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct CyCloudContainerTask {
    _marker: (),
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct CyCloudNativeTask {
    user_name: BoundedVec<u8, ConstU32<20>>,
}

#[derive(
	PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking,
)]
pub struct CyCloudVmTask {
    user_name: BoundedVec<u8, ConstU32<20>>,
}

///Detailed information about a specific task.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TaskInfo<AccountId, BlockNumber> {
	pub task_owner: AccountId,                  // Who scheduled the task.
	pub create_block: BlockNumber,              // Block when created.
	pub time_elapsed: Option<BlockNumber>,      // Time consumed.
	pub average_cpu_percentage_use: Option<u8>, // CPU usage.
	pub task_kind: TaskKind,       // New: Logical kind (NeuroZK or OpenInference).
	pub result: Option<BoundedVec<u8, ConstU32<500>>>, // Final result (optional).
	pub compute_hours_deposit: Option<u32>,     // Deposit paid upfront.
	pub consume_compute_hours: Option<u32>,     // How much was actually consumed.
	pub task_status: TaskStatusType,            // Current lifecycle status.
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