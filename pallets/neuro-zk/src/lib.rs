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
	oracle::OracleKey,
};
use frame_support::{pallet_prelude::IsType, sp_runtime::RuntimeDebug, BoundedVec};
use frame_support::{traits::Get};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerificationResult<BlockNumber> {
	pub is_accepted: bool,
	pub block: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_timestamp::Config + pallet_edge_connect::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;

		/// The percentage of active oracle entries needed to determine proof acceptance status
		#[pallet::constant]
		type AcceptanceThreshold: Get<u8>;

		/// Number of feed values needed to reach consensus on a proof
		#[pallet::constant]
		type AggregateLength: Get<u32>;

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

	/// Stores the verification results for each neuro-zk task.
	/// The result is provided by different oracle feeders, and the data is collected and aggregated to calculate
	/// the overall status for each task.
	///
	/// - The storage key is a `TaskId`, which uniquely identifies the task.
	/// - The value is a bounded vector of `VerificationResult`, which contains the verification status.
	#[pallet::storage]
	pub type VerificationResultsPerProof<T: Config> = StorageMap<
		_,
		Twox64Concat,
		TaskId,
		BoundedVec<VerificationResult<BlockNumberFor<T>>, T::AggregateLength>,
		ValueQuery,
	>;

	/// Tracks whether a specific oracle provider has submitted zk proof data during the current period.
	/// This is used to prevent multiple submissions from the same oracle feeder within a period.
	///
	/// - The key is a tuple of the oracle feeder's account and required proof info `(T::AccountId, TaskId)`.
	/// - The value is a boolean indicating whether the oracle has already submitted data.
	#[pallet::storage]
	pub type SubmittedPerProof<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, TaskId), bool, ValueQuery>;

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
		InvalidTaskType,

		/// The task for which the proof was requested does not exist.
		TaskDoesNotExist,
	}

	// This block defines the dispatchable functions (calls) for the pallet.
	// Dispatchable functions are the publicly accessible functions that users or other pallets
	// can call to interact with the pallet. Each function has a weight and requires the user
	// to sign the transaction unless specified otherwise.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Requests a nzk proof from the given task
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::request_proof())]
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
						RequestedProofs::<T>::insert(task_id, ProofVerificationStage::Requested);
					},
					_ => {
						return Err(Error::<T>::InvalidTaskType.into());
					}
				}
			} else {
				return Err(Error::<T>::TaskDoesNotExist.into());
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
		#[pallet::weight(<T as pallet::Config>::WeightInfo::submit_proof())]
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

	impl<T: Config> Pallet<T> {
		/// Finalizes proof verification for a given task_id by reaching consensius based on `T::AcceptanceThreshold`
		fn finalize_verification(task_id: TaskId) {
			let current_block = <frame_system::Pallet<T>>::block_number();

			let verification_results = VerificationResultsPerProof::<T>::get(task_id);
			let accepted_count = verification_results.iter().filter(|r| r.is_accepted).count();
			let total = verification_results.len();
			if total == 0 {
				log::error!(target: "nzk", "No verification results found for task_id: {:?}, cannot finalize verification", task_id);
				return
			}
			let accepted_percentage = (accepted_count * 100 / total) as u8;

			let is_accepted = accepted_percentage >= T::AcceptanceThreshold::get();

			Self::update_nzk_task(task_id, is_accepted, current_block);

			log::info!(
				target: "nzk",
				"Finalizing verification for task_id: {:?}, accepted_percentage: {:?}, accepted_count: {:?}, total: {:?}, is_accepted: {:?}",
				task_id,
				accepted_percentage,
				accepted_count,
				total,
				is_accepted
			)
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
					log::error!(target: "nzk", "Neuro-ZK data is missing in the task");
				}
			} else {
				log::error!(target: "nzk", "Neuro-ZK task not found for the given task id");
			}
		}

		/// Handles new data received from the oracle feeder
		pub fn on_new_data(
			who: &T::AccountId,
			key: &TaskId,
			value: &bool,
		) {
			if T::NzkTaskInfoHandler::get_nzk_task(*key).is_none() {
				log::error!(
					target: "nzk",
					"No task with the ID: {:?}",
					key
				);
				return;
			}
			if SubmittedPerProof::<T>::get((who, *key)) {
				log::error!(
					target: "nzk",
					"A value for this proof was already submitted by: {:?}",
					who
				);
				return;
			}

			let mut should_finalize = false;

			VerificationResultsPerProof::<T>::mutate(*key, |results_vec| {
				let verification_result = VerificationResult {
					is_accepted: *value,	
					block: <frame_system::Pallet<T>>::block_number(),
				};

				match results_vec.try_push(verification_result) {
					Ok(()) => {
						log::info!(
							target: "nzk",
							"Successfully push status instance value for proof. \
							Task: {:?}, Value was submitted by: {:?}",
							key,who
						);

						if results_vec.len() as u32 == T::AggregateLength::get() {
							should_finalize = true;
							log::info!(
								target: "nzk",
								"Finalizing verification for proof: {:?}",
								key
							)
						}

						SubmittedPerProof::<T>::set((who, *key), true);
					}
					Err(_) => {
						log::error!(
						target: "nzk",
								"Failed to push proof verification result due to exceeded capacity. \
								Value was submitted by: {:?}",
								who
						);
					}
				}
			});

			if should_finalize {
				Self::finalize_verification(*key);
			}
		}
	}
}
