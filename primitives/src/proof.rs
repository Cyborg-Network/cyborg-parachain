use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;
use crate::task::TaskId;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum ProofVerificationStatus {
	Pending,
	Verified,
	Rejected,
}

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen)]
pub struct VerificationStatusAggregator<AccountId> {
	pub pending: BoundedVec<AccountId, ConstU32<10000>>,
	pub verified: BoundedVec<AccountId, ConstU32<10000>>,
	pub rejected: BoundedVec<AccountId, ConstU32<10000>>,
}

pub type VerifiedTasks<MaxVerificationsPerAcc> = BoundedVec<(TaskId, bool), MaxVerificationsPerAcc>;

/// Response from the off-chain worker containing the task id and its verification result
pub type OcwResponse<MaxTasksPerBlock> = BoundedVec<(TaskId, bool), MaxTasksPerBlock>;

//TODO! Figure out the correct type for EZKL proofs
pub type Proof = u8;
