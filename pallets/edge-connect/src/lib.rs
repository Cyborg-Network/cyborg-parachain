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

pub use cyborg_primitives::worker::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::sp_runtime::Saturating;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_timestamp as timestamp;
	use scale_info::prelude::vec::Vec;

	// The `Config` trait defines the configuration for this pallet. It specifies the types and parameters
	// that the pallet depends on and provides flexibility to the runtime in how it implements these
	// requirements.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// A helper function providing a default value for worker IDs.
	#[pallet::type_value]
	pub fn WorkerCountDefault() -> WorkerId {
		0
	}

	// A helper function providing a default value for worker reputations.
	#[pallet::type_value]
	pub fn WorkerReputationDefault<T: Config>() -> WorkerReputation<BlockNumberFor<T>> {
		WorkerReputation::default()
	}

	/// AccountWorkers Information, Storage map for associating an account ID with a worker ID. If no worker exists, the query returns None.
	/// Keeps track of workerIds per account if any
	#[pallet::storage]
	#[pallet::getter(fn account_workers)]
	pub type AccountWorkers<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WorkerId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn suspended_workers)]
	pub type SuspendedWorkers<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		(BlockNumberFor<T>, SuspensionReason),
		OptionQuery,
	>;

	/// Worker Cluster information, Storage map to keep track of detailed worker cluster information for each (account ID, worker ID) pair.
	#[pallet::storage]
	pub type WorkerClusters<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
		OptionQuery,
	>;

	/// Execultable Worker information, Storage map to keep track of detailed worker cluster information for each (account ID, worker ID) pair.
	#[pallet::storage]
	pub type ExecutableWorkers<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
		OptionQuery,
	>;

	/// The `Event` enum contains the various events that can be emitted by this pallet.
	/// Events are emitted when significant actions or state changes happen in the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a new worker is successfully registered.
		///
		/// - `creator`: The account ID of the worker's creator.
		/// - `worker`: A tuple containing the account ID of the worker owner and the worker ID.
		/// - `domain`: The domain associated with the
		WorkerRegistered {
			creator: T::AccountId,
			worker: (T::AccountId, WorkerId),
			domain: Domain,
		},

		/// Event emitted when a worker is removed from the system.
		///
		/// - `creator`: The account ID of the worker's creator.
		/// - `worker_id`: The ID of the worker that was removed.
		WorkerRemoved {
			creator: T::AccountId,
			worker_id: WorkerId,
		},

		/// Event emitted when a worker's status is updated (e.g., toggling visibility).
		///
		/// - `creator`: The account ID of the worker's creator.
		/// - `worker_id`: The ID of the worker whose status was updated.
		/// - `worker_status`: The new status of the worker, either active or inactive.
		WorkerStatusUpdated {
			creator: T::AccountId,
			worker_id: WorkerId,
			worker_status: WorkerStatusType,
		},

		/// Event emitted when a worker is penalized
		WorkerPenalized {
			worker: (T::AccountId, WorkerId),
			penalty: i32,
			reason: PenaltyReason,
		},

		/// Event emitted when a worker is suspended
		WorkerSuspended {
			worker: (T::AccountId, WorkerId),
			until_block: BlockNumberFor<T>,
		},

		/// Event emitted when a worker is put under review
		WorkerUnderReview {
			worker: (T::AccountId, WorkerId),
			reason: SuspensionReason,
		},

		/// Event emitted when a worker is banned
		WorkerBanned {
			worker: (T::AccountId, WorkerId),
			reason: SuspensionReason,
		},

		/// Event emitted when a worker is unsuspended
		WorkerUnsuspended { worker: (T::AccountId, WorkerId) },
	}

	#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub enum PenaltyReason {
		TaskRejection,
		FalseCompletion,
		LateResponse,
		SpamAttempt,
		Other,
	}

	/// The `Error` enum contains all possible errors that can occur when interacting with this pallet.
	/// These errors will be returned in the `DispatchResult` when a function call fails.
	#[pallet::error]
	pub enum Error<T> {
		/// Error indicating that either the IP address or the domain was missing when attempting to register a worker.
		WorkerRegisterMissingIpOrDomain,
		/// Error indicating that the worker already exists and cannot be registered again.
		WorkerExists,
		/// Error indicating that the worker does not exist in the system when trying to perform actions (e.g., removal or status update).
		WorkerDoesNotExist,
		/// Worker is suspended and cannot perform actions.
		WorkerSuspended,
		/// Worker reputation is too low
		InsufficientReputation,
	}

	// This block defines the dispatchable functions (calls) for the pallet.
	// Dispatchable functions are the publicly accessible functions that users or other pallets
	// can call to interact with the pallet. Each function has a weight and requires the user
	// to sign the transaction unless specified otherwise.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers a Worker with either a domain and initialize it with an inactive status.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_worker())]
		pub fn register_worker(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			domain: Domain,
			latitude: Latitude,
			longitude: Longitude,
			ram: RamBytes,
			storage: StorageBytes,
			cpu: CpuCores,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			let api = WorkerAPI { domain };
			let worker_keys = AccountWorkers::<T>::get(creator.clone());
			let worker_location = Location {
				latitude,
				longitude,
			};
			let worker_specs = WorkerSpecs { ram, storage, cpu };

			//TODO: There needs to be a proper id mechanism to avoid loops and the increment id system
			match worker_keys {
				Some(keys) => {
					for id in 0..=keys {
						// Get the Worker associated with the creator and worker_id
						if let Some(worker) = WorkerClusters::<T>::get((creator.clone(), id)) {
							if worker_type == cyborg_primitives::worker::WorkerType::Docker {
								// Check if the API matches and throw an error if it does
								ensure!(api != worker.api, Error::<T>::WorkerExists);
							}
						}
						if let Some(worker) = ExecutableWorkers::<T>::get((creator.clone(), id)) {
							if worker_type == cyborg_primitives::worker::WorkerType::Executable {
								// Check if the API matches and throw an error if it does
								ensure!(api != worker.api, Error::<T>::WorkerExists);
							}
						}
					}
				}
				None => {}
			}

			let worker_id: WorkerId = match AccountWorkers::<T>::get(creator.clone()) {
				Some(id) => {
					AccountWorkers::<T>::insert(creator.clone(), id + 1);
					id + 1
				}
				None => {
					AccountWorkers::<T>::insert(creator.clone(), 0);
					0
				}
			};

			let blocknumber = <frame_system::Pallet<T>>::block_number();
			let worker = Worker {
				id: worker_id.clone(),
				owner: creator.clone(),
				location: worker_location,
				specs: worker_specs,
				reputation: WorkerReputation::<BlockNumberFor<T>>::default(),
				start_block: blocknumber.clone(),
				status: WorkerStatusType::Inactive,
				status_last_updated: blocknumber.clone(),
				api: api,
				last_status_check: timestamp::Pallet::<T>::get(),
			};

			// update storage
			AccountWorkers::<T>::insert(creator.clone(), worker_id.clone());

			match worker_type {
				cyborg_primitives::worker::WorkerType::Docker => {
					WorkerClusters::<T>::insert((creator.clone(), worker_id.clone()), worker.clone());
				}
				cyborg_primitives::worker::WorkerType::Executable => {
					ExecutableWorkers::<T>::insert((creator.clone(), worker_id.clone()), worker.clone());
				}
			}

			// Emit an event.
			Self::deposit_event(Event::WorkerRegistered {
				creator: creator.clone(),
				worker: (worker.owner, worker.id),
				domain: worker.api.domain,
			});

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// Remove a worker from storage an deactivates it
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_worker())]
		pub fn remove_worker(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			worker_id: WorkerId,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			match worker_type {
				WorkerType::Docker => {
					ensure!(
						WorkerClusters::<T>::get((creator.clone(), worker_id)) != None,
						Error::<T>::WorkerDoesNotExist
					);

					// update storage
					WorkerClusters::<T>::remove((creator.clone(), worker_id));
				}
				WorkerType::Executable => {
					ensure!(
						ExecutableWorkers::<T>::get((creator.clone(), worker_id)) != None,
						Error::<T>::WorkerDoesNotExist
					);

					// update storage
					ExecutableWorkers::<T>::remove((creator.clone(), worker_id));
				}
			}

			// Emit an event.
			Self::deposit_event(Event::WorkerRemoved { creator, worker_id });

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// Switches the visibility of a worker between active and inactive.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::toggle_worker_visibility())]
		pub fn toggle_worker_visibility(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			worker_id: WorkerId,
			visibility: bool,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			let worker_status = if visibility {
				WorkerStatusType::Active
			} else {
				WorkerStatusType::Inactive
			};

			match worker_type {
				WorkerType::Docker => {
					WorkerClusters::<T>::mutate((creator.clone(), worker_id), |worker_option| {
						if let Some(worker) = worker_option {
							worker.status = worker_status;
							worker.last_status_check = timestamp::Pallet::<T>::get();

							Self::deposit_event(Event::WorkerStatusUpdated {
								creator,
								worker_id,
								worker_status: worker.status.clone(),
							});
							Ok(())
						} else {
							Err(Error::<T>::WorkerDoesNotExist)
						}
					})
				}
				WorkerType::Executable => {
					ExecutableWorkers::<T>::mutate((creator.clone(), worker_id), |worker_option| {
						if let Some(worker) = worker_option {
							worker.status = worker_status;
							worker.last_status_check = timestamp::Pallet::<T>::get();

							Self::deposit_event(Event::WorkerStatusUpdated {
								creator,
								worker_id,
								worker_status: worker.status.clone(),
							});
							Ok(())
						} else {
							Err(Error::<T>::WorkerDoesNotExist)
						}
					})
				}
			}?;

			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::penalize_worker())]
		pub fn penalize_worker(
			origin: OriginFor<T>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			worker_type: WorkerType,
			penalty: i32,
			reason: PenaltyReason,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::apply_penalty(&(worker_owner, worker_id), &worker_type, penalty, reason)?;

			Ok(())
		}

		/// Manually suspend a worker (root only)
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::suspend_worker())]
		pub fn suspend_worker(
			origin: OriginFor<T>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			worker_type: WorkerType,
			blocks: BlockNumberFor<T>,
			reason: SuspensionReason,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::suspend_workers(&(worker_owner, worker_id), &worker_type, blocks, reason)
		}

		/// Manually ban a worker (root only)
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::ban_worker())]
		pub fn ban_worker(
			origin: OriginFor<T>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			worker_type: WorkerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::ban_workers(&(worker_owner, worker_id), worker_type, reason)
		}

		/// Lift suspension from a worker (root only)
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unsuspend_worker())]
		pub fn unsuspend_worker(
			origin: OriginFor<T>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			worker_type: WorkerType,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::lift_suspension(&(worker_owner, worker_id), &worker_type)
		}
	}

	impl<T: Config> Pallet<T> {
		// Helper Function to retrieve all active workers from storage.
		// Filters workers based on their status (active or inactive).
		pub fn get_active_workers() -> Option<
			Vec<(
				(T::AccountId, WorkerId),
				Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
			)>,
		> {
			let workers = WorkerClusters::<T>::iter()
				.filter(|&(_, ref worker)| worker.status == WorkerStatusType::Active)
				.collect::<Vec<_>>();

			if workers.is_empty() {
				None
			} else {
				Some(workers)
			}
		}

		pub fn is_registered_miner(account: &T::AccountId) -> bool {
			AccountWorkers::<T>::contains_key(account)
		}

		/// Apply penalty to a worker's reputation
		fn apply_penalty(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
			penalty: i32,
			reason: PenaltyReason,
		) -> DispatchResult {
			let mut worker = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			// Apply penalty
			worker.reputation.score = worker.reputation.score.saturating_sub(penalty);
			worker.reputation.violations += 1;
			worker.reputation.last_updated = Some(<frame_system::Pallet<T>>::block_number());

			// Automatic suspension triggers
			if worker.reputation.score < 30 {
				// Severe penalty - suspend for 1000 blocks (~4 hours at 6s/block)
				Self::suspend_workers(
					worker_key,
					&worker_type.clone(),
					1000u32.into(),
					SuspensionReason::ReputationThreshold,
				)?;
			} else if worker.reputation.score < 50 {
				// Moderate penalty - put under review
				Self::put_worker_under_review(
					worker_key,
					&worker_type.clone(),
					SuspensionReason::ReputationThreshold,
				)?;
			} else if worker.reputation.violations > 10 {
				// Too many violations - review
				Self::put_worker_under_review(
					worker_key,
					&worker_type.clone(),
					SuspensionReason::RepeatedTaskFailures,
				)?;
			}

			// Update storage if not suspended
			if worker.status != WorkerStatusType::Suspended {
				match worker_type {
					WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
					WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
				}
			}

			Self::deposit_event(Event::WorkerPenalized {
				worker: worker_key.clone(),
				penalty,
				reason,
			});

			Ok(())
		}

		/// Check if worker can perform actions
		pub fn check_worker_status(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: WorkerType,
		) -> DispatchResult {
			let worker = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			// Check if suspended
			if worker.status == WorkerStatusType::Suspended {
				if <frame_system::Pallet<T>>::block_number() < worker.status_last_updated {
					return Err(Error::<T>::WorkerSuspended.into());
				} else {
					// Auto-unsuspend if suspension period is over
					let mut worker = worker.clone();
					worker.status = WorkerStatusType::Inactive;
					match worker_type {
						WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
						WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
					}
				}
			}

			// Check reputation
			if worker.reputation.score < 50 {
				return Err(Error::<T>::InsufficientReputation.into());
			}

			Ok(())
		}

		/// Suspend a worker with a specific reason and duration
		fn suspend_workers(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
			blocks: BlockNumberFor<T>,
			reason: SuspensionReason,
		) -> DispatchResult {
			let mut worker = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let suspension_end = current_block.saturating_add(blocks);

			// Update worker status
			worker.status = WorkerStatusType::Suspended;
			worker.status_last_updated = suspension_end;
			worker.reputation.suspension_count += 1;

			// Update storage
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}

			// Record suspension
			SuspendedWorkers::<T>::insert(worker_key, (suspension_end, reason.clone()));

			Self::deposit_event(Event::WorkerSuspended {
				worker: worker_key.clone(),
				until_block: suspension_end,
			});

			Ok(())
		}

		/// Put worker under review
		fn put_worker_under_review(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			let mut worker = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			worker.status = WorkerStatusType::Inactive; // Can't accept new tasks
			worker.reputation.review_count += 1;

			// Update storage
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}

			Self::deposit_event(Event::WorkerUnderReview {
				worker: worker_key.clone(),
				reason,
			});

			Ok(())
		}

		/// Ban a worker permanently
		fn ban_workers(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: WorkerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			// Remove from active workers
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::remove(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::remove(worker_key),
			}

			Self::deposit_event(Event::WorkerBanned {
				worker: worker_key.clone(),
				reason,
			});

			Ok(())
		}

		/// Lift suspension from a worker
		fn lift_suspension(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
		) -> DispatchResult {
			let mut worker = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			// Only proceed if actually suspended
			if worker.status != WorkerStatusType::Suspended {
				return Ok(());
			}

			// Update worker status
			worker.status = WorkerStatusType::Inactive;
			worker.status_last_updated = <frame_system::Pallet<T>>::block_number();

			// Update storage
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}

			// Remove from suspended workers
			SuspendedWorkers::<T>::remove(worker_key);

			Self::deposit_event(Event::WorkerUnsuspended {
				worker: worker_key.clone(),
			});

			Ok(())
		}
	}

	impl<T: Config + timestamp::Config>
		WorkerInfoHandler<T::AccountId, WorkerId, BlockNumberFor<T>, T::Moment> for Pallet<T>
	{
		// Implementation of the WorkerInfoHandler trait, which provides methods for accessing worker cluster information.
		fn get_worker_cluster(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
		) -> Option<Worker<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
		}

		// Implementation of the WorkerInfoHandler trait, which provides methods for updating worker cluster information.
		fn update_worker_cluster(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
			worker: Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
		) {
			match worker_type {
				WorkerType::Docker => {
					WorkerClusters::<T>::insert(worker_key, worker);
				}
				WorkerType::Executable => {
					ExecutableWorkers::<T>::insert(worker_key, worker);
				}
			}
		}
	}
}
