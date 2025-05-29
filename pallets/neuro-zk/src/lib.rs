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
pub use cyborg_primitives::{
	zkml::*,
	task::{TaskId, NzkTaskInfoHandler, TaskKind, ZkProof},
};
use frame_support::{pallet_prelude::IsType, sp_runtime::RuntimeDebug, BoundedVec};
use frame_support::{traits::Get, LOG_TARGET};
use orml_traits::{CombineData, OnNewData};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerificationResult<BlockNumber> {
	pub is_accepted: bool,
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
pub struct VerificationStatusPercentages<BlockNumber> {
	pub is_accepted: u8,
	pub last_block_processed: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

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

		/// The percentage of active oracle entries needed to determine proof acceptance status
		#[pallet::constant]
		type ThresholdAcceptanceStatus: Get<u8>;

		/// Maximum number of status entries by unique oracle feeders for a worker per period
		#[pallet::constant]
		type MaxAggregateParamLength: Get<u32>;

		/// Updates Task Status for Task Management
		type NzkTaskInfoHandler: NzkTaskInfoHandler<
			Self::AccountId,
			TaskId,
			BlockNumberFor<Self>,
		>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Keeps track of if a proof is already being processed, so that the user can't request more
	#[pallet::storage]
	pub type RequestedProofs<T: Config> = StorageMap<_, Twox64Concat, TaskId, ProofVerificationStage, OptionQuery>;

	/// Stores the last block number that the pallet processed for clearing data.
	/// This is used to track the last time data was aggregated and cleared by the pallet's hooks.
	#[pallet::storage]
	pub type LastClearedBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// Stores the verification results for each neuro-zk task over a specific period.
	/// The result is provided by different oracle feeders, and the data is collected and aggregated to calculate
	/// the overall status for each task.
	///
	/// - The storage key is a `TaskId`, which uniquely identifies the task.
	/// - The value is a bounded vector of `VerificationResult`, which contains the verification status over time.
	#[pallet::storage]
	pub type VerificationResultsPerPeriod<T: Config> = StorageMap<
		_,
		Twox64Concat,
		TaskId,
		BoundedVec<VerificationResult<BlockNumberFor<T>>, T::MaxAggregateParamLength>,
		ValueQuery,
	>;

	/// Tracks whether a specific oracle provider has submitted zk proof data during the current period.
	/// This is used to prevent multiple submissions from the same oracle feeder within a period.
	///
	/// - The key is a tuple of the oracle feeder's account and required proof info `(T::AccountId, TaskId)`.
	/// - The value is a boolean indicating whether the oracle has already submitted data.
	#[pallet::storage]
	pub type SubmittedPerPeriod<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, TaskId), bool, ValueQuery>;

	/// Stores the resulting acceptance percentage for each proof after aggregation.
	/// This is calculated by taking the status data submitted during the period and determining the
	/// percentage of .
	///
	/// - The key is a `TaskId`, representing the task with the associated proof.
	/// - The value is `ProcessStatusPercentages`, which contains the percentages and the block number of the last processed status.
	#[pallet::storage]
	pub type ResultingVerificationStatusPercentages<T: Config> = StorageMap<
		_,
		Twox64Concat,
		TaskId,
		VerificationStatusPercentages<BlockNumberFor<T>>,
		ValueQuery,
	>;

	/// Stores the final verification status for each task based on the percentage thresholds.
	/// The final status is determined based on the configured threshold values for proof acceptance.
	///
	/// - The key is a `TaskId`, representing the task.
	/// - The value is a `bool`, which represents if the proof was accepted or rejected.
	#[pallet::storage]
	pub type ResultingProofAcceptanceStatus<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, bool, ValueQuery>;

	/// The `Event` enum contains the various events that can be emitted by this pallet.
	/// Events are emitted when significant actions or state changes happen in the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when a proof has been requested.
		NzkProofRequested {
			requesting_account: T::AccountId,
			task_id: TaskId,
		},

		/// Emitted when a proof has been submitted.
		NzkProofSubmitted {
			submitting_miner: T::AccountId,
			task_id: TaskId,
		},

		/// Emitted when a proof has successfully been verified after aggregation.
		NzkProofVerified {
			task_id: TaskId,
			last_block_processed: BlockNumberFor<T>,
		},

		/// Emitted when a proof has been rejected after aggregation.
		NzkProofRejected {
			task_id: TaskId,
			last_block_processed: BlockNumberFor<T>,
		},

		/// Event emitted when the last block is updated after clearing data for the current period.
		/// This indicates that data from the oracle has been successfully processed and cleared for the given block range.
		///
		/// - `block_number`: The block number at which the clearing occurred.
		LastBlockUpdated { block_number: BlockNumberFor<T> },
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The proof has already been requested and needs to be verified before a new one can be requested.
		ProofAlreadyRequested,	

		/// The proof has already been submitted and another one cannot be submitted until a new one is requested.
		ProofAlreadySubmitted,

		/// The miner tried to submit a proof that wasn't requested.
		ProofNotRequested,

		/// The task for which the proof was requested is a non-neuro-zk task type.
		InvalidTaskType
	}

	// This block defines the dispatchable functions (calls) for the pallet.
	// Dispatchable functions are the publicly accessible functions that users or other pallets
	// can call to interact with the pallet. Each function has a weight and requires the user
	// to sign the transaction unless specified otherwise.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Requests a nzk proof from the given task
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn request_proof(
			origin: OriginFor<T>,
			//TODO keep track of who requests the proof in case it is not the gatekeeper
			//TODO requesting_account: T::AccountId,
			task_id: TaskId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if a proof has already been requested
			ensure!(
				!RequestedProofs::<T>::contains_key(&task_id), 
				Error::<T>::ProofAlreadyRequested
			);

			// Check if the task is a NeuroZK task
			if let Some(task) = T::NzkTaskInfoHandler::get_nzk_task(task_id) {
				match task.task_kind {
					TaskKind::NeuroZK => {
						// Insert the task_id into the RequestedProofs map
						RequestedProofs::<T>::insert(task_id, ProofVerificationStage::Requested);
					},
					_ => {
						return Err(Error::<T>::InvalidTaskType.into());
					}
				}
			}

			// Emit an event.
			Self::deposit_event(Event::NzkProofRequested {
				requesting_account: who,
				task_id: task_id,
			});

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// Submits a nzk proof from the given task
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn submit_proof(
			origin: OriginFor<T>,
			//TODO keep track of who requests the proof in case it is not the gatekeeper
			//TODO requesting_account: T::AccountId,
			task_id: TaskId,
			proof: ZkProof,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if the proof is actually requested and not already submitted
			if let Some(requested_proof) = RequestedProofs::<T>::get(task_id) {
				if requested_proof != ProofVerificationStage::Requested {
					return Err(Error::<T>::ProofAlreadySubmitted.into());
				}
			} else {
				return Err(Error::<T>::ProofNotRequested.into());
			}

			// Check if the task is a NeuroZK task and set the proof
			if let Some(mut task) = T::NzkTaskInfoHandler::get_nzk_task(task_id) {
				match task.task_kind {
					TaskKind::NeuroZK => {
						if let Some(ref mut nzk_data) = task.nzk_data {
							nzk_data.zk_proof = Some(proof);
							T::NzkTaskInfoHandler::update_nzk_task(task_id, task);
						}
						// Insert the task_id into the RequestedProofs map
						RequestedProofs::<T>::insert(task_id, ProofVerificationStage::Pending);
					},
					_ => {
						return Err(Error::<T>::InvalidTaskType.into());
					}
				}
			}

			// Emit an event.
			Self::deposit_event(Event::NzkProofSubmitted {
				submitting_miner: who,
				task_id: task_id,
			});

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}

	/// This hook function is called at the end of each block to process worker status data for a given period.
	/// It checks whether the current block number exceeds the last cleared block by the maximum block range period.
	/// If so, it aggregates the proof verification data for the period, clears outdated data, and updates the veriification status.
	/// It also logs the result of the clearing process and emits an event when the last block is updated.
	/// This hook calculates storage values in this pallet updated by the oracle per MaxBlockRangePeriod
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(now: BlockNumberFor<T>) {
			if LastClearedBlock::<T>::get() + T::MaxBlockRangePeriod::get() <= now {
				Self::process_aggregate_data_for_period();
				let clear_result_a = SubmittedPerPeriod::<T>::clear(500, None);
				let clear_result_b = VerificationResultsPerPeriod::<T>::clear(500, None);
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
								"Clearing map result for WorkerStatusEntriesPerPeriod: {:?}",
								clear_result_b.deconstruct()
				);
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_aggregate_data_for_period() {
			for (key_task_id, verification_status_vec) in VerificationResultsPerPeriod::<T>::iter() {
				let mut total_accepted: u32 = 0;
				verification_status_vec
					.iter()
					.for_each(|value: &VerificationResult<BlockNumberFor<T>>| {
						total_accepted += if value.is_accepted { 100 } else { 0 };
					});
				let is_accepted = (total_accepted / verification_status_vec.len() as u32) as u8;
				let current_block = <frame_system::Pallet<T>>::block_number();
				let verification_status_percentages = VerificationStatusPercentages {
					is_accepted,
					last_block_processed: current_block,
				};
				ResultingVerificationStatusPercentages::<T>::set(&key_task_id, verification_status_percentages);

				let accepted_status = is_accepted >= T::ThresholdAcceptanceStatus::get();
				ResultingProofAcceptanceStatus::<T>::set(
					key_task_id,
					accepted_status
				);
				Self::update_nzk_task(
					key_task_id,
					accepted_status,
					current_block,
				);
			}
		}

		/// Sends updated nzk task info to pallets that implement T::NzkTaskHandler and emits an event
		fn update_nzk_task(
			task_id: TaskId,
			is_accepted: bool,
			last_block_processed: BlockNumberFor<T>,
		) {
			if let Some(mut task) = T::NzkTaskInfoHandler::get_nzk_task(task_id) {
				if let Some(ref mut nzk_data) = task.nzk_data {
					nzk_data.last_proof_accepted = Some((is_accepted, last_block_processed));

					T::NzkTaskInfoHandler::update_nzk_task(task_id, task);

					if is_accepted {
						Self::deposit_event(Event::NzkProofVerified {
							task_id,
							last_block_processed,
						});
					} else {
						Self::deposit_event(Event::NzkProofRejected {
							task_id,
							last_block_processed,
						});
					}
				} else {
					log::warn!("Neuro-ZK data is missing in the task");
				}
			} else {
				log::warn!("Neuro-ZK task not found for the given task id");
			}
		}
	}

	/// Data from the oracle first enters into this pallet through this trait implementation and updates this pallet's storage
	impl<T: Config> OnNewData<T::AccountId, TaskId, bool>
		for Pallet<T>
	{
		fn on_new_data(
			who: &T::AccountId,
			key: &TaskId,
			value: &bool,
		) {
			if T::NzkTaskInfoHandler::get_nzk_task(*key).is_none() {
				log::error!(
					target: LOG_TARGET,
					"No worker registed by this key: {:?}",
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
			VerificationResultsPerPeriod::<T>::mutate(key, |results_vec| {
				match results_vec.try_push(VerificationResult {
					is_accepted: *value,
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

	impl<T: Config + orml_oracle::Config> CombineData<TaskId, TimestampedValue<T>>
		for Pallet<T>
	{
		fn combine_data(
			_key: &TaskId,
			_values: Vec<TimestampedValue<T>>,
			_prev_value: Option<TimestampedValue<T>>,
		) -> Option<TimestampedValue<T>> {
			None
		}
	}
}
