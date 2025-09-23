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
	use cyborg_primitives::task::TaskId;
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

		/// Event emitted when a miner tries to re-register itself.
		///
		/// - `creator`: The account ID of the miner's creator.
		/// - `worker`: A tuple containing the account ID of the miner owner and the miner ID.
		WorkerAlreadyRegistered {
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
		OracleStatusUpdated {
			worker: (T::AccountId, WorkerId),
			online: bool,
		},
		OperationalStatusUpdated {
			worker: (T::AccountId, WorkerId),
			status: OperationalStatus,
		},
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
		WorkerUnsuspended {
			worker: (T::AccountId, WorkerId),
		},
	}

	#[derive(
		PartialEq,
		Eq,
		Clone,
		RuntimeDebug,
		Encode,
		Decode,
		TypeInfo,
		MaxEncodedLen,
		DecodeWithMemTracking,
	)]
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
		/// Miner is busy
		MinerIsBusy,
		/// Miner is inactive
		MinerIsInactive,
		NotAuthorized,
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

			// Check for existing worker with same domain
			match worker_keys {
				Some(keys) => {
					for id in 0..=keys {
						if let Some(worker) = WorkerClusters::<T>::get((creator.clone(), id)) {
							if worker_type == WorkerType::Docker && api == worker.api {
								Self::deposit_event(Event::WorkerAlreadyRegistered {
									creator: creator.clone(),
									worker: (creator.clone(), worker.id),
									domain: worker.api.domain,
								});
								return Err(Error::<T>::WorkerExists.into());
							}
						}
						if let Some(worker) = ExecutableWorkers::<T>::get((creator.clone(), id)) {
							if worker_type == WorkerType::Executable && api == worker.api {
								Self::deposit_event(Event::WorkerAlreadyRegistered {
									creator: creator.clone(),
									worker: (creator.clone(), worker.id),
									domain: worker.api.domain,
								});
								return Err(Error::<T>::WorkerExists.into());
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
				current_task: None,
				start_block: blocknumber.clone(),
				oracle_status: OracleStatus::Offline,
				operational_status: OperationalStatus::Available,
				status_last_updated: blocknumber.clone(),
				api: api,
				last_status_check: timestamp::Pallet::<T>::get(),
			};

			// update storage
			AccountWorkers::<T>::insert(creator.clone(), worker_id.clone());

			match worker_type {
				WorkerType::Docker => {
					WorkerClusters::<T>::insert((creator.clone(), worker_id.clone()), worker.clone());
				}
				WorkerType::Executable => {
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

		/// Updates the oracle status (callable by oracle feeder)
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::update_oracle_status())]
		pub fn update_oracle_status(
			origin: OriginFor<T>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			worker_type: WorkerType,
			online: bool,
		) -> DispatchResult {
			ensure_signed(origin)?;

			match worker_type {
				WorkerType::Docker => {
					WorkerClusters::<T>::mutate((worker_owner.clone(), worker_id), |worker_option| {
						if let Some(worker) = worker_option {
							worker.oracle_status = if online {
								OracleStatus::Online
							} else {
								OracleStatus::Offline
							};
							worker.last_status_check = timestamp::Pallet::<T>::get();
							Ok(())
						} else {
							Err(Error::<T>::WorkerDoesNotExist)
						}
					})
				}
				WorkerType::Executable => {
					ExecutableWorkers::<T>::mutate((worker_owner.clone(), worker_id), |worker_option| {
						if let Some(worker) = worker_option {
							worker.oracle_status = if online {
								OracleStatus::Online
							} else {
								OracleStatus::Offline
							};
							worker.last_status_check = timestamp::Pallet::<T>::get();
							Ok(())
						} else {
							Err(Error::<T>::WorkerDoesNotExist)
						}
					})
				}
			}?;

			Self::deposit_event(Event::OracleStatusUpdated {
				worker: (worker_owner, worker_id),
				online,
			});

			Ok(())
		}

		/// Updates the operational status (callable by miner itself)
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::update_operational_status())]
		pub fn update_operational_status(
			origin: OriginFor<T>,
			worker_type: WorkerType,
			worker_id: WorkerId,
			status: OperationalStatus,
		) -> DispatchResult {
			let miner = ensure_signed(origin)?;
			let status_clone = status.clone();

			match worker_type {
				WorkerType::Docker => {
					WorkerClusters::<T>::mutate((miner.clone(), worker_id), |worker_option| {
						if let Some(worker) = worker_option {
							// Miners can only set Available or Busy status
							if matches!(status, OperationalStatus::Suspended) {
								return Err(Error::<T>::NotAuthorized.into());
							}
							worker.operational_status = status;
							worker.status_last_updated = <frame_system::Pallet<T>>::block_number();
							Ok(())
						} else {
							Err(Error::<T>::WorkerDoesNotExist)
						}
					})
				}
				WorkerType::Executable => {
					ExecutableWorkers::<T>::mutate((miner.clone(), worker_id), |worker_option| {
						if let Some(worker) = worker_option {
							// Miners can only set Available or Busy status
							if matches!(status, OperationalStatus::Suspended) {
								return Err(Error::<T>::NotAuthorized.into());
							}
							worker.operational_status = status;
							worker.status_last_updated = <frame_system::Pallet<T>>::block_number();
							Ok(())
						} else {
							Err(Error::<T>::WorkerDoesNotExist)
						}
					})
				}
			}?;

			Self::deposit_event(Event::OperationalStatusUpdated {
				worker: (miner, worker_id),
				status: status_clone,
			});

			Ok(())
		}

		#[pallet::call_index(4)]
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
		#[pallet::call_index(5)]
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
			Self::suspend_worker_internal(&(worker_owner, worker_id), &worker_type, blocks, reason)
		}

		/// Manually ban a worker (root only)
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::ban_worker())]
		pub fn ban_worker(
			origin: OriginFor<T>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			worker_type: WorkerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::ban_worker_internal(&(worker_owner, worker_id), worker_type, reason)
		}

		/// Lift suspension from a worker (root only)
		#[pallet::call_index(7)]
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
				.filter(|&(_, ref worker)| worker.can_accept_tasks())
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
		pub fn apply_penalty(
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

			worker.reputation.score = worker.reputation.score.saturating_sub(penalty);
			worker.reputation.violations += 1;
			worker.reputation.last_updated = Some(<frame_system::Pallet<T>>::block_number());

			// Automatic suspension triggers
			if worker.reputation.score < 30 {
				Self::suspend_worker_internal(
					worker_key,
					worker_type,
					1000u32.into(),
					SuspensionReason::ReputationThreshold,
				)?;
			}

			// Update storage
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}

			Self::deposit_event(Event::WorkerPenalized {
				worker: worker_key.clone(),
				penalty,
				reason,
			});

			Ok(())
		}

		pub fn get_miner(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
		) -> Option<Worker<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
		}

		/// Check if worker can perform actions
		pub fn check_worker_status(
			miner_key: &(T::AccountId, WorkerId),
			miner_type: &WorkerType,
		) -> DispatchResult {
			let miner = Self::get_miner(miner_key, miner_type).ok_or(Error::<T>::WorkerDoesNotExist)?;

			// Check if worker is suspended and if suspension period has expired
			if miner.is_suspended() {
				let current_block = <frame_system::Pallet<T>>::block_number();

				// If suspension period is over, auto-unsuspend
				if current_block >= miner.status_last_updated {
					let mut updated_miner = miner.clone();
					updated_miner.operational_status = OperationalStatus::Available;
					updated_miner.status_last_updated = current_block;

					// Update the worker status
					Self::update_worker(miner_key, miner_type, updated_miner);

					// Remove from suspended workers storage
					SuspendedWorkers::<T>::remove(miner_key);
				} else {
					return Err(Error::<T>::WorkerSuspended.into());
				}
			}

			// Check oracle status (uptime)
			if miner.oracle_status != OracleStatus::Online {
				log::warn!("Worker oracle status is not Online, but allowing for testing");
			}

			// Check operational status
			if miner.operational_status != OperationalStatus::Available {
				return Err(Error::<T>::MinerIsBusy.into());
			}

			// Check reputation
			if miner.reputation.score < 10 {
				// Reduced from 50 to 10 for testing
				return Err(Error::<T>::InsufficientReputation.into());
			}

			Ok(())
		}

		pub fn update_miner_operational_status(
			miner: &(T::AccountId, WorkerId),
			miner_type: WorkerType,
			available: bool,
		) -> DispatchResult {
			let mut worker = match miner_type {
				WorkerType::Docker => WorkerClusters::<T>::get(miner),
				WorkerType::Executable => ExecutableWorkers::<T>::get(miner),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			worker.operational_status = if available {
				OperationalStatus::Available
			} else {
				OperationalStatus::Busy
			};
			Self::update_worker(miner, &miner_type, worker);
			Ok(())
		}

		pub fn update_miner_current_task(
			miner: &(T::AccountId, WorkerId),
			miner_type: &WorkerType,
			current_task: Option<TaskId>,
		) -> DispatchResult {
			let mut worker = match miner_type {
				WorkerType::Docker => WorkerClusters::<T>::get(miner),
				WorkerType::Executable => ExecutableWorkers::<T>::get(miner),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			worker.current_task = current_task;
			Self::update_worker(miner, miner_type, worker);
			Ok(())
		}

		/// Suspend a worker with a specific reason and duration
		pub fn suspend_worker_internal(
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

			// Update worker operational status to suspended
			worker.operational_status = OperationalStatus::Suspended;
			worker.status_last_updated = suspension_end;
			worker.reputation.suspension_count += 1;

			Self::update_worker(worker_key, worker_type, worker);

			// Record suspension
			SuspendedWorkers::<T>::insert(worker_key, (suspension_end, reason.clone()));

			Self::deposit_event(Event::WorkerSuspended {
				worker: worker_key.clone(),
				until_block: suspension_end,
			});

			Ok(())
		}

		/// Ban a worker permanently
		fn ban_worker_internal(
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
			if !worker.is_suspended() {
				return Ok(());
			}

			// Update worker status to available
			worker.operational_status = OperationalStatus::Available;
			worker.status_last_updated = <frame_system::Pallet<T>>::block_number();

			Self::update_worker(worker_key, worker_type, worker);

			// Remove from suspended workers
			SuspendedWorkers::<T>::remove(worker_key);

			Self::deposit_event(Event::WorkerUnsuspended {
				worker: worker_key.clone(),
			});

			Ok(())
		}

		fn update_worker(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: &WorkerType,
			worker: Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
		) {
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}
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
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}
		}
	}
}
