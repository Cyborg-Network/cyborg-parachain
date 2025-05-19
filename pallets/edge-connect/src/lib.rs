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
use frame_support::traits::ConstU32;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
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

	/// Storage map to index workers by their location (latitude, longitude)
	#[pallet::storage]
	pub type WorkersByLocation<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Latitude,
		Twox64Concat,
		Longitude,
		BoundedVec<(T::AccountId, WorkerId), ConstU32<100>>, // Max 100 workers per location
		ValueQuery,
	>;

	/// Storage map to index all workers by owner
	#[pallet::storage]
	pub type WorkersByOwner<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<WorkerId, ConstU32<100>>, // Max 100 workers per owner
		ValueQuery,
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
		TooManyWorkersAtLocation,
		TooManyWorkersForOwner,
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

			// Check for existing workers with same API
			match worker_keys {
				Some(keys) => {
					for id in 0..=keys {
						if let Some(worker) = WorkerClusters::<T>::get((creator.clone(), id)) {
							if worker_type == cyborg_primitives::worker::WorkerType::Docker {
								ensure!(api != worker.api, Error::<T>::WorkerExists);
							}
						}
						if let Some(worker) = ExecutableWorkers::<T>::get((creator.clone(), id)) {
							if worker_type == cyborg_primitives::worker::WorkerType::Executable {
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

			// Update storage
			AccountWorkers::<T>::insert(creator.clone(), worker_id.clone());

			match worker_type {
				cyborg_primitives::worker::WorkerType::Docker => {
					WorkerClusters::<T>::insert((creator.clone(), worker_id.clone()), worker.clone());
				}
				cyborg_primitives::worker::WorkerType::Executable => {
					ExecutableWorkers::<T>::insert((creator.clone(), worker_id.clone()), worker.clone());
				}
			}

			// Update indices
			WorkersByLocation::<T>::try_append(
				worker.location.latitude,
				worker.location.longitude,
				(creator.clone(), worker_id.clone()),
			)
			.map_err(|_| Error::<T>::TooManyWorkersAtLocation)?;
			WorkersByOwner::<T>::try_append(creator.clone(), worker_id.clone())
				.map_err(|_| Error::<T>::TooManyWorkersForOwner)?;

			Self::deposit_event(Event::WorkerRegistered {
				creator: creator.clone(),
				worker: (worker.owner, worker.id),
				domain: worker.api.domain,
			});

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

			// Clean up indices before removing worker
			if let Some(worker) = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get((creator.clone(), worker_id)),
				WorkerType::Executable => ExecutableWorkers::<T>::get((creator.clone(), worker_id)),
			} {
				// Remove from location index
				WorkersByLocation::<T>::mutate(
					worker.location.latitude,
					worker.location.longitude,
					|workers| {
						if let Some(pos) = workers
							.iter()
							.position(|w| w == &(creator.clone(), worker_id))
						{
							workers.swap_remove(pos);
						}
					},
				);

				// Remove from owner index
				WorkersByOwner::<T>::mutate(&creator, |worker_ids| {
					if let Some(pos) = worker_ids.iter().position(|id| id == &worker_id) {
						worker_ids.swap_remove(pos);
					}
				});
			}

			match worker_type {
				WorkerType::Docker => {
					ensure!(
						WorkerClusters::<T>::get((creator.clone(), worker_id)) != None,
						Error::<T>::WorkerDoesNotExist
					);
					WorkerClusters::<T>::remove((creator.clone(), worker_id));
				}
				WorkerType::Executable => {
					ensure!(
						ExecutableWorkers::<T>::get((creator.clone(), worker_id)) != None,
						Error::<T>::WorkerDoesNotExist
					);
					ExecutableWorkers::<T>::remove((creator.clone(), worker_id));
				}
			}

			Self::deposit_event(Event::WorkerRemoved { creator, worker_id });

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

			Self::apply_penalty(&(worker_owner, worker_id), worker_type, penalty, reason)?;

			Ok(())
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

		/// Get workers at a specific location
		pub fn get_workers_by_location(
			latitude: Latitude,
			longitude: Longitude,
		) -> BoundedVec<(T::AccountId, WorkerId), ConstU32<100>> {
			WorkersByLocation::<T>::get(latitude, longitude)
		}

		/// Get workers owned by a specific account
		pub fn get_workers_by_owner(owner: T::AccountId) -> BoundedVec<WorkerId, ConstU32<100>> {
			WorkersByOwner::<T>::get(owner)
		}

		/// Get detailed worker information by location
		pub fn get_worker_details_by_location(
			latitude: Latitude,
			longitude: Longitude,
		) -> Vec<Worker<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			WorkersByLocation::<T>::get(latitude, longitude)
				.into_inner() 
				.into_iter()
				.filter_map(|(owner, id)| {
					WorkerClusters::<T>::get((owner.clone(), id))
						.or_else(|| ExecutableWorkers::<T>::get((owner, id)))
				})
				.collect()
		pub fn is_registered_miner(account: &T::AccountId) -> bool {
			AccountWorkers::<T>::contains_key(account)
		}

		/// Apply penalty to a worker's reputation
		fn apply_penalty(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: WorkerType,
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

			// If reputation drops below threshold, suspend the worker
			if worker.reputation.score < 30 {
				worker.status = WorkerStatusType::Suspended;
				// Suspend for 1000 blocks (~4 hours at 6s/block)
				worker.status_last_updated = <frame_system::Pallet<T>>::block_number() + 1000u32.into();
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
