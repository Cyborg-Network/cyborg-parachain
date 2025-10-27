use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;
use crate::task::TaskId;

pub type MinerId = u64;

pub type Domain = BoundedVec<u8, ConstU32<128>>;

pub type Latitude = i32;

pub type Longitude = i32;

pub type RamBytes = u64;

pub type StorageBytes = u64;

pub type CpuCores = u16;

/// An enum that is used to differentiate between the different kinds of Miners that are
/// registered on the cyborg parachain.
#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, PartialOrd, Ord, DecodeWithMemTracking)]
pub enum MinerType {
	Cloud,
	Edge,
}

/// TODO:
#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen, DecodeWithMemTracking)]
pub enum MinerStatusType {
	Active,
	Busy,
	Inactive,
	Suspended,
}

/// TODO:
#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MinerAPI {
	pub domain: Domain,
}

/// TODO:
#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Location {
	pub latitude: Latitude,
	pub longitude: Longitude,
}

/// TODO:
#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MinerSpecs {
	pub ram: RamBytes,
	pub storage: StorageBytes,
	pub cpu: CpuCores,
}

/// TODO:
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Miner<AccountId, BlockNumber, TimeStamp> {
	pub id: MinerId,
	pub owner: AccountId,
	pub location: Location,
	pub specs: MinerSpecs,
	pub reputation: MinerReputation<BlockNumber>,
	pub current_task: Option<TaskId>,
	pub start_block: BlockNumber,
	pub status: MinerStatusType,
	pub status_last_updated: BlockNumber,
	pub api: MinerAPI,
	pub last_status_check: TimeStamp,
}

/// TODO:
pub trait MinerInfoHandler<AccountId, MinerId, BlockNumber, TimeStamp> {
	fn get_miner(
		miner_key: &(AccountId, MinerId),
		miner_type: &MinerType,
	) -> Option<Miner<AccountId, BlockNumber, TimeStamp>>;
	fn update_miner(
		miner_key: &(AccountId, MinerId),
		miner_type: &MinerType,
		miner: Miner<AccountId, BlockNumber, TimeStamp>,
	);
}

/// TODO:
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen, Copy)]
pub struct MinerReputation<BlockNumber> {
	pub score: i32,
	pub last_updated: Option<BlockNumber>,
	pub violations: u32,
	pub successful_tasks: u32,
	pub suspension_count: u32,
	pub review_count: u32,
}

impl<BlockNumber> Default for MinerReputation<BlockNumber> {
	fn default() -> Self {
		Self {
			score: 100,
			last_updated: None,
			violations: 0,
			successful_tasks: 0,
			suspension_count: 0,
			review_count: 0,
		}
	}
}

/// TODO:
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum SuspicionLevel {
	Review,
	Suspension,
	Ban,
}

/// TODO:
#[derive(
	PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen, DecodeWithMemTracking,
)]
pub enum SuspensionReason {
	RepeatedTaskFailures,
	SpamBehavior,
	MaliciousActivity,
	ReputationThreshold,
	ManualOverride,
	TaskConfirmationTimeout,
}
