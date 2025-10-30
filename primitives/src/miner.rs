use crate::task::TaskId;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;


// pub type MinerId = u64;

pub type MaxUuidLen = ConstU32<64>;

pub type MinerId = BoundedVec<u8, MaxUuidLen>;

pub type Domain = BoundedVec<u8, ConstU32<128>>;

pub type Latitude = i32;

pub type Longitude = i32;

pub type RamBytes = u64;

pub type StorageBytes = u64;

pub type CpuCores = u16;

/// An enum that is used to differentiate between the different kinds of Miners that are
/// registered on the cyborg parachain.
#[derive(
	PartialEq,
	Eq,
	Clone,
	Decode,
	Encode,
	TypeInfo,
	Debug,
	MaxEncodedLen,
	PartialOrd,
	Ord,
	DecodeWithMemTracking,
)]
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
/// Status controlled by the oracle feeder - reflects uptime and availability
#[derive(
	PartialEq,
	Eq,
	Clone,
	Decode,
	Encode,
	TypeInfo,
	Debug,
	MaxEncodedLen,
	PartialOrd,
	Ord,
	DecodeWithMemTracking,
)]
pub enum OracleStatus {
	Online,  // Worker is online and responsive (set by oracle)
	Offline, // Worker is not responding (set by oracle)
}

/// Status controlled by the miner itself
#[derive(
	PartialEq,
	Eq,
	Clone,
	Decode,
	Encode,
	TypeInfo,
	Debug,
	MaxEncodedLen,
	PartialOrd,
	Ord,
	DecodeWithMemTracking,
)]
pub enum OperationalStatus {
	Available, // Miner is available for new tasks (set by miner)
	Busy,      // Miner is currently processing a task (set by miner)
	Suspended, // Miner is suspended (set by system via reputation penalties)
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
	// Two independent status types
	pub oracle_status: OracleStatus, // Set by oracle feeder (uptime)
	pub operational_status: OperationalStatus, // Set by miner itself + system (operational state)
	pub status_last_updated: BlockNumber,
	pub api: MinerAPI,
	pub last_status_check: TimeStamp,
}

// Helper method to check if miner can accept tasks
impl<AccountId, BlockNumber, TimeStamp> Miner<AccountId, BlockNumber, TimeStamp> {
	pub fn can_accept_tasks(&self) -> bool {
		self.oracle_status == OracleStatus::Online
			&& self.operational_status == OperationalStatus::Available
	}

	pub fn is_suspended(&self) -> bool {
		self.operational_status == OperationalStatus::Suspended
	}

	pub fn is_online_and_available(&self) -> bool {
		self.oracle_status == OracleStatus::Online
			&& self.operational_status == OperationalStatus::Available
	}

	pub fn is_online(&self) -> bool {
		self.oracle_status == OracleStatus::Online
	}

	pub fn is_operational(&self) -> bool {
		!self.is_suspended() && self.operational_status != OperationalStatus::Suspended
	}

	pub fn is_eligible_for_tasks(&self) -> bool {
		self.can_accept_tasks() && self.reputation.score >= 50
	}
}

pub trait MinerInfoHandler<AccountId, MinerId, BlockNumber, TimeStamp> {
	fn get_miner(
		miner_key: &MinerId,
		miner_type: &MinerType,
	) -> Option<Miner<AccountId, BlockNumber, TimeStamp>>;
	fn update_miner(
		miner_key: &MinerId,
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
