use crate::task::TaskId;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

pub type WorkerId = u64;

pub type Domain = BoundedVec<u8, ConstU32<128>>;

pub type Latitude = i32;

pub type Longitude = i32;

pub type RamBytes = u64;

pub type StorageBytes = u64;

pub type CpuCores = u16;

/// An enum that is used to differentiate between the different kinds of workers that are
/// registered on the cyborg parachain. There is no differentiation between the ZK Worker and the
/// Executable Worker, as the executable worker will be able to execute ZK Tasks
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
pub enum WorkerType {
	Docker,
	Executable,
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

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct WorkerAPI {
	pub domain: Domain,
}

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Location {
	pub latitude: Latitude,
	pub longitude: Longitude,
}

#[derive(Default, PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct WorkerSpecs {
	pub ram: RamBytes,
	pub storage: StorageBytes,
	pub cpu: CpuCores,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Worker<AccountId, BlockNumber, TimeStamp> {
	pub id: WorkerId,
	pub owner: AccountId,
	pub location: Location,
	pub specs: WorkerSpecs,
	pub reputation: WorkerReputation<BlockNumber>,
	pub current_task: Option<TaskId>,
	pub start_block: BlockNumber,

	// Two independent status types
	pub oracle_status: OracleStatus, // Set by oracle feeder (uptime)
	pub operational_status: OperationalStatus, // Set by miner itself + system (operational state)

	pub status_last_updated: BlockNumber,
	pub api: WorkerAPI,
	pub last_status_check: TimeStamp,
}

// Helper method to check if worker can accept tasks
impl<AccountId, BlockNumber, TimeStamp> Worker<AccountId, BlockNumber, TimeStamp> {
	pub fn can_accept_tasks(&self) -> bool {
		self.oracle_status == OracleStatus::Online
			&& self.operational_status == OperationalStatus::Available
	}

	pub fn is_suspended(&self) -> bool {
		self.operational_status == OperationalStatus::Suspended
	}
}

pub trait WorkerInfoHandler<AccountId, WorkerId, BlockNumber, TimeStamp> {
	fn get_worker_cluster(
		worker_key: &(AccountId, WorkerId),
		worker_type: &WorkerType,
	) -> Option<Worker<AccountId, BlockNumber, TimeStamp>>;
	fn update_worker_cluster(
		worker_key: &(AccountId, WorkerId),
		worker_type: &WorkerType,
		worker: Worker<AccountId, BlockNumber, TimeStamp>,
	);
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen, Copy)]
pub struct WorkerReputation<BlockNumber> {
	pub score: i32,
	pub last_updated: Option<BlockNumber>,
	pub violations: u32,
	pub successful_tasks: u32,
	pub suspension_count: u32,
	pub review_count: u32,
}

impl<BlockNumber> Default for WorkerReputation<BlockNumber> {
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

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum SuspicionLevel {
	Review,
	Suspension,
	Ban,
}

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
