use codec::{Decode, Encode, MaxEncodedLen, DecodeWithMemTracking};
use sp_runtime::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use crate::task::TaskId;

pub type MaxTasksPerBlock = ConstU32<1>;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum ProofVerificationStatus {
	Init,
	Requested,
	Pending,
	Verified,
	Rejected,
}

pub type ZkInput = BoundedVec<u8, ConstU32<1000000>>;
pub type ZkSettings = BoundedVec<u8, ConstU32<1000000>>;
pub type ZkVerifyingKey = BoundedVec<u8, ConstU32<1000000>>;
pub type ZkProof = BoundedVec<u8, ConstU32<1000000>>;

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen)]
pub struct VerificationStatusAggregator<AccountId> {
	pub pending: BoundedVec<AccountId, ConstU32<10000>>,
	pub verified: BoundedVec<AccountId, ConstU32<10000>>,
	pub rejected: BoundedVec<AccountId, ConstU32<10000>>,
}

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen, DecodeWithMemTracking, PartialEq, Debug)]
pub struct NeuroZkTaskSubmissionDetails {
	pub zk_input: ZkInput,
	pub zk_settings: ZkSettings,
	pub zk_verifying_key: ZkVerifyingKey,
}

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen)]
pub struct NeuroZkTaskInfo {
	// network.onnx and proving key archive location is stored in pallet-task-managment
	pub zk_input: ZkInput,
	pub zk_settings: ZkSettings,
	pub zk_verifying_key: ZkVerifyingKey,
	pub zk_proof: Option<ZkProof>,
	pub status: ProofVerificationStatus,
}

pub type VerifiedTasks<MaxVerificationsPerAcc> = BoundedVec<(TaskId, bool), MaxVerificationsPerAcc>;

/// Response from the off-chain worker containing the task id and its verification result
pub type OcwResponse<MaxTasksPerBlock> = BoundedVec<(TaskId, bool), MaxTasksPerBlock>;