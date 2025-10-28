#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

use codec::{Decode, Encode, MaxEncodedLen};
use cyborg_primitives::miner::{MinerId, MinerInfoHandler, MinerType};
use frame_support::{pallet_prelude::IsType, sp_runtime::RuntimeDebug, BoundedVec};
use frame_support::{traits::Get, LOG_TARGET};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct StatusInstance<BlockNumber> {
	pub is_online: bool,
	pub is_available: bool,
	pub block: BlockNumber,
}

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
)]
pub struct ProcessStatusPercentages<BlockNumber> {
	pub online: u8,
	pub available: u8,
	pub last_block_processed: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use cyborg_primitives::oracle::{OracleMinerFormat, ProcessStatus};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_edge_connect::OracleStatus;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_timestamp::Config + pallet_edge_connect::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// /// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;

		/// Maximum number of blocks or block range used to calculate average status
		#[pallet::constant]
		type MaxBlockRangePeriod: Get<BlockNumberFor<Self>>;

		/// The percentage of active oracle entries needed to determine online status for miner
		#[pallet::constant]
		type ThresholdUptimeStatus: Get<u8>;

		/// Maximum number of status entries by unique oracle feeders for a miner per period
		#[pallet::constant]
		type MaxAggregateParamLength: Get<u32>;

		/// Updates Miner Status for Edge Connect
		type MinerInfoHandler: MinerInfoHandler<
			Self::AccountId,
			MinerId,
			BlockNumberFor<Self>,
			Self::Moment,
		>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Stores the last block number that the pallet processed for clearing data.
	/// This is used to track the last time data was aggregated and cleared by the pallet's hooks.
	#[pallet::storage]
	pub type LastClearedBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Stores the status entries (online/offline, available/unavailable) for each miner over a specific period.
	/// The status is provided by different oracle feeders, and the data is collected and aggregated to calculate
	/// the overall status for each miner.
	///
	/// - The storage key is a tuple of `(T::AccountId, MinerId)`, which uniquely identifies the miner.
	/// - The value is a bounded vector of `StatusInstance`, which contains the miner's status over time.
	#[pallet::storage]
	pub type MinerStatusEntriesPerPeriod<T: Config> = StorageMap<
		_,
		Twox64Concat,
		OracleMinerFormat,
		BoundedVec<StatusInstance<BlockNumberFor<T>>, T::MaxAggregateParamLength>,
		ValueQuery,
	>;

	/// Tracks whether a specific oracle provider has submitted miner status data during the current period.
	/// This is used to prevent multiple submissions from the same oracle provider within a period.
	///
	/// - The key is a tuple of the oracle provider's account and required miner info `(T::AccountId, OracleMinerFormat)`.
	/// - The value is a boolean indicating whether the oracle has already submitted data.
	#[pallet::storage]
	pub type SubmittedPerPeriod<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, OracleMinerFormat), bool, ValueQuery>;

	/// Stores the resulting percentage status (online and available) for each miner after aggregation.
	/// This is calculated by taking the status data submitted during the period and determining the
	/// percentage of time the miner was online and available.
	///
	/// - The key is `(T::AccountId, MinerId)`, representing the miner.
	/// - The value is `ProcessStatusPercentages`, which contains the percentages and the block number of the last processed status.
	#[pallet::storage]
	pub type ResultingMinerStatusPercentages<T: Config> = StorageMap<
		_,
		Twox64Concat,
		OracleMinerFormat,
		ProcessStatusPercentages<BlockNumberFor<T>>,
		ValueQuery,
	>;

	/// Stores the final status (online/offline and available/unavailable) for each miner based on the percentage thresholds.
	/// The final status is determined based on the configured threshold values for uptime.
	///
	/// - The key is `(T::AccountId, MinerId)`, representing the miner.
	/// - The value is `ProcessStatus`, which contains the final online and available status for the miner.
	#[pallet::storage]
	pub type ResultingMinerStatus<T: Config> =
		StorageMap<_, Twox64Concat, OracleMinerFormat, ProcessStatus, ValueQuery>;

	/// The `Event` enum contains the various events that can be emitted by this pallet.
	/// Events are emitted when significant actions or state changes happen in the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when the miner status is updated based on aggregated data from the oracle.
		/// This provides the new online and availability status for the miner and the block number where the status was last updated.
		///
		/// - `miner`: A tuple containing the miner's account ID and the miner ID.
		/// - `online`: A boolean indicating whether the miner is online.
		/// - `available`: A boolean indicating whether the miner is available.
		/// - `last_block_processed`: The block number at which the miner's status was last updated.
		UpdateFromAggregatedMinerInfo {
			miner: MinerId,
			online: bool,
			available: bool,
			last_block_processed: BlockNumberFor<T>,
		},

		/// Event emitted when the last block is updated after clearing data for the current period.
		/// This indicates that data from the oracle has been successfully processed and cleared for the given block range.
		///
		/// - `block_number`: The block number at which the clearing occurred.
		LastBlockUpdated { block_number: BlockNumberFor<T> },
	}

	/// This hook function is called at the end of each block to process miner status data for a given period.
	/// It checks whether the current block number exceeds the last cleared block by the maximum block range period.
	/// If so, it aggregates the miner status data for the period, clears outdated data, and updates the miner status.
	/// It also logs the result of the clearing process and emits an event when the last block is updated.
	/// This hook calculates storage values in this pallet updated by the oracle per MaxBlockRangePeriod
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(now: BlockNumberFor<T>) {
			if LastClearedBlock::<T>::get() + T::MaxBlockRangePeriod::get() <= now {
				Self::process_aggregate_data_for_period();
				let clear_result_a = SubmittedPerPeriod::<T>::clear(500, None);
				let clear_result_b = MinerStatusEntriesPerPeriod::<T>::clear(500, None);
				if clear_result_a.maybe_cursor.is_none() && clear_result_b.maybe_cursor.is_none() {
					LastClearedBlock::<T>::set(now);
					Self::deposit_event(Event::LastBlockUpdated { block_number: now });
				}
				log::info!(
						target: LOG_TARGET,
								"Clearing map result for SubmittedPerPeriod: {:?}",
								clear_result_a.deconstruct()
				);
				log::info!(
						target: LOG_TARGET,
								"Clearing map result for MinerStatusEntriesPerPeriod: {:?}",
								clear_result_b.deconstruct()
				);
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_aggregate_data_for_period() {
			for (key_miner, value_status_vec) in MinerStatusEntriesPerPeriod::<T>::iter() {
				let mut total_online: u32 = 0;
				let mut total_available: u32 = 0;
				value_status_vec
					.iter()
					.for_each(|value: &StatusInstance<BlockNumberFor<T>>| {
						total_online += if value.is_online { 100 } else { 0 };
						total_available += if value.is_available { 100 } else { 0 };
					});
				let online = (total_online / value_status_vec.len() as u32) as u8;
				let available = (total_available / value_status_vec.len() as u32) as u8;
				let current_block = <frame_system::Pallet<T>>::block_number();
				let process_status_percentages = ProcessStatusPercentages {
					online,
					available,
					last_block_processed: current_block,
				};
				ResultingMinerStatusPercentages::<T>::set(&key_miner, process_status_percentages);

				// Update miner statuses
				let online_status = online >= T::ThresholdUptimeStatus::get();
				let available_status = available >= T::ThresholdUptimeStatus::get();
				ResultingMinerStatus::<T>::set(
					key_miner.clone(),
					ProcessStatus {
						online: online_status,
						available: available_status,
					},
				);
				Self::update_miner_clusters(
					key_miner.id,
					key_miner.miner_type,
					online_status,
					available_status,
					current_block,
				);
			}
		}
		/// sends updated miner info to pallets that implement T::MinerClusterHandler and emits an event
		fn update_miner_clusters(
			key_miner: MinerId,
			miner_type: MinerType,
			online: bool,
			available: bool,
			last_block_processed: BlockNumberFor<T>,
		) {
			if let Some(mut miner_cluster) = T::MinerInfoHandler::get_miner(&key_miner, &miner_type) {
				// Update oracle_status based on online status
				miner_cluster.oracle_status = if online {
					OracleStatus::Online
				} else {
					OracleStatus::Offline
				};

				miner_cluster.status_last_updated = last_block_processed;

				T::MinerInfoHandler::update_miner(&key_miner, &miner_type, miner_cluster);

				Self::deposit_event(Event::UpdateFromAggregatedMinerInfo {
					miner: key_miner,
					online,
					available,
					last_block_processed,
				});
			} else {
				log::warn!("Miner cluster not found for the given account and miner_id.");
			}
		}

		pub fn on_new_data(
			who: &T::AccountId,
			key: &OracleMinerFormat,
			value: &ProcessStatus,
		) {
			if T::MinerInfoHandler::get_miner(&key.id, &key.miner_type).is_none() {
				log::error!(
					target: LOG_TARGET,
					"No miner registed by this key: {:?}",
					key
				);
				return;
			}
			if SubmittedPerPeriod::<T>::get((who, key)) {
				log::error!(
					target: LOG_TARGET,
					"A value for this period was already submitted by: {:?}",
					who
				);
				return;
			}
			MinerStatusEntriesPerPeriod::<T>::mutate(key, |status_vec| {
				match status_vec.try_push(StatusInstance {
					is_online: value.online,
					is_available: value.available,
					block: <frame_system::Pallet<T>>::block_number(),
				}) {
					Ok(()) => {
						log::info!(
							target: LOG_TARGET,
							"Successfully push status instance value for period. \
							Value was submitted by: {:?}",
							who
						);
					}
					Err(_) => {
						log::error!(
						target: LOG_TARGET,
								"Failed to push status instance value due to exceeded capacity. \
								Value was submitted by: {:?}",
								who
						);
					}
				}
			});
			SubmittedPerPeriod::<T>::set((who, key), true);
		}
	}

	/*
	/// Data from the oracle first enters into this pallet through this trait implementation and updates this pallet's storage
	impl<T: Config> OnNewData<T::AccountId, OracleMinerFormat<T::AccountId>, ProcessStatus>
		for Pallet<T>
	{
		fn on_new_data(
			who: &T::AccountId,
			key: &OracleMinerFormat<T::AccountId>,
			value: &ProcessStatus,
		) {
			if T::MinerInfoHandler::get_miner_cluster(&key.id, &key.miner_type).is_none() {
				log::error!(
					target: LOG_TARGET,
					"No miner registed by this key: {:?}",
					key
				);
				return;
			}
			if SubmittedPerPeriod::<T>::get((who, key)) {
				log::error!(
					target: LOG_TARGET,
					"A value for this period was already submitted by: {:?}",
					who
				);
				return;
			}
			MinerStatusEntriesPerPeriod::<T>::mutate(key, |status_vec| {
				match status_vec.try_push(StatusInstance {
					is_online: value.online,
					is_available: value.available,
					block: <frame_system::Pallet<T>>::block_number(),
				}) {
					Ok(()) => {
						log::info!(
							target: LOG_TARGET,
							"Successfully push status instance value for period. \
							Value was submitted by: {:?}",
							who
						);
					}
					Err(_) => {
						log::error!(
						target: LOG_TARGET,
								"Failed to push status instance value due to exceeded capacity. \
								Value was submitted by: {:?}",
								who
						);
					}
				}
			});
			SubmittedPerPeriod::<T>::set((who, key), true);
		}
	 }*/

	/*
	impl<T: Config + orml_oracle::Config> CombineData<(T::AccountId, MinerId), TimestampedValue<T>>
		for Pallet<T>
	{
		fn combine_data(
			_key: &(T::AccountId, MinerId),
			_values: Vec<TimestampedValue<T>>,
			_prev_value: Option<TimestampedValue<T>>,
		) -> Option<TimestampedValue<T>> {
			None
		}
	}
	*/
}
