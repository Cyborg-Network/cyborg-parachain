use crate::task::TaskId;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, traits::Time, BoundedVec};
use orml_oracle::Config;
use orml_traits;
use scale_info::TypeInfo;
use sp_std::vec::Vec;

pub type MaxTasksPerBlock = ConstU32<1>;

pub type TimestampedValue<T, I = ()> =
	orml_oracle::TimestampedValue<bool, <<T as orml_oracle::Config<I>>::Time as Time>::Moment>;

/// This type represents the STAGE of the zkml proof verification process, NOT the STATUS, which is stored in the task-management pallet
#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum ProofVerificationStage {
	Requested,
	Pending,
	Finalized,
}

pub type ZkInput = BoundedVec<u8, ConstU32<1000000>>;
pub type ZkSettings = BoundedVec<u8, ConstU32<1000000>>;
pub type ZkVerifyingKey = BoundedVec<u8, ConstU32<1000000>>;
pub type ZkProof = BoundedVec<u8, ConstU32<1000000>>;

/*
#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen)]
pub struct VerificationStatusAggregator<AccountId> {
	pub pending: BoundedVec<AccountId, ConstU32<10000>>,
	pub verified: BoundedVec<AccountId, ConstU32<10000>>,
	pub rejected: BoundedVec<AccountId, ConstU32<10000>>,
}
*/

#[derive(Clone, Decode, Encode, TypeInfo, MaxEncodedLen, PartialEq, Debug)]
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
	pub zk_proofs: Option<ZkProof>,
	pub status: ProofVerificationStage,
}

pub type VerifiedTasks<MaxVerificationsPerAcc> = BoundedVec<(TaskId, bool), MaxVerificationsPerAcc>;

/// Response from the node-side verifying-daemon containing the task id and its verification result
pub type NodeProofResponse<MaxTasksPerBlock> = BoundedVec<(TaskId, bool), MaxTasksPerBlock>;

/// A dummy implementation of `CombineData` trait that does nothing.
pub struct DummyCombineData<T, I = ()>(PhantomData<(T, I)>);
impl<T: Config<I>, I> orml_traits::CombineData<TaskId, TimestampedValue<T, I>>
	for DummyCombineData<T, I>
where
	<T as Config<I>>::Time: frame_support::traits::Time,
{
	fn combine_data(
		_key: &TaskId,
		_values: Vec<TimestampedValue<T, I>>,
		_prev_value: Option<TimestampedValue<T, I>>,
	) -> Option<TimestampedValue<T, I>> {
		None
	}
}
