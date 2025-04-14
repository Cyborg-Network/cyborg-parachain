use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum ProofVerificationStatus {
	Verified,
	Rejected,
	Pending,
}

pub type VerificationResult = bool;

//TODO! Figure out the correct type for EZKL proofs
pub type Proof = u8;