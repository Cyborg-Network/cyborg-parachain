use crate::task::TaskId;
use crate::worker::{WorkerId, WorkerType};
use frame_support::{pallet_prelude::*, traits::Time};
use orml_oracle::Config;
use orml_traits;
use scale_info::prelude::string::String;
use sp_std::vec::Vec;

/// Interface for fetching metrics and Logs.
///
/// **NOTE:** This is just a temporary interface, and will be replaced with a proper oracle which will
/// provide metrics and logs data of a connected cluster
pub trait MetricsAndLogs {
	/// get metrics
	fn get_metrics() -> StringAPI;
}

pub type StringAPI = String;

#[derive(
	Default,
	Encode,
	Decode,
	MaxEncodedLen,
	Clone,
	Copy,
	Debug,
	Ord,
	PartialOrd,
	PartialEq,
	Eq,
	TypeInfo,
	DecodeWithMemTracking
)]
pub struct ProcessStatus {
	pub online: bool,
	pub available: bool,
	// TaskResultHash: Option<H256>,
}

#[derive(
	Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialOrd, Ord, DecodeWithMemTracking
)]
pub enum OracleKey<AccountId> {
	Miner(OracleWorkerFormat<AccountId>),
	NzkProofResult(TaskId),
}

#[derive(
	Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, PartialOrd, Ord, DecodeWithMemTracking
)]
pub enum OracleValue {
	MinerStatus(ProcessStatus),
	ZkProofResult(bool),
}

#[derive(Encode, Decode, MaxEncodedLen, Clone, Debug, PartialEq, Eq, TypeInfo, PartialOrd, Ord, DecodeWithMemTracking)]
pub struct OracleWorkerFormat<AccoundId> {
	pub id: (AccoundId, WorkerId),
	pub worker_type: WorkerType,
}

#[derive(Encode, Decode, MaxEncodedLen, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum MachineId {
	Id(u64),
}

#[derive(Encode, Decode, MaxEncodedLen, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
pub enum ProcessId {
	Process(u64, MachineId),
}

pub type TimestampedValue<T, I = ()> =
	orml_oracle::TimestampedValue<OracleValue, <<T as orml_oracle::Config<I>>::Time as Time>::Moment>;

/// A dummy implementation of `CombineData` trait that does nothing.
pub struct DummyCombineData<T, I = ()>(PhantomData<(T, I)>);
impl<T: Config<I>, I> orml_traits::CombineData<OracleKey<T::AccountId>, TimestampedValue<T, I>>
	for DummyCombineData<T, I>
where
	<T as Config<I>>::Time: frame_support::traits::Time,
{
	fn combine_data(
		_key: &OracleKey<T::AccountId>,
		_values: Vec<TimestampedValue<T, I>>,
		_prev_value: Option<TimestampedValue<T, I>>,
	) -> Option<TimestampedValue<T, I>> {
		None
	}
}
