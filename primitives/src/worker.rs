use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, sp_runtime::RuntimeDebug, BoundedVec};
use scale_info::TypeInfo;

pub type WorkerId = u64;

pub type Domain = BoundedVec<u8, ConstU32<128>>;

pub type Latitude = i32;

pub type Longitude = i32;

pub type RamBytes = u64;

pub type StorageBytes = u64;

pub type CpuCores = u16;

pub type WorkerReputation = u8;

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum WorkerStatusType {
	Active,
	Busy,
	Inactive,
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
	pub reputation: WorkerReputation,
	pub start_block: BlockNumber,
	pub status: WorkerStatusType,
	pub status_last_updated: BlockNumber,
	pub api: WorkerAPI,
	pub last_status_check: TimeStamp,
}

pub trait WorkerInfoHandler<AccountId, WorkerId, BlockNumber, TimeStamp> {
	fn get_worker_cluster(
		worker_key: &(AccountId, WorkerId),
	) -> Option<Worker<AccountId, BlockNumber, TimeStamp>>;
	fn update_worker_cluster(
		worker_key: &(AccountId, WorkerId),
		worker: Worker<AccountId, BlockNumber, TimeStamp>,
	);
}
