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
	pub fn WorkerReputationDefault() -> WorkerReputation {
		0
	}

	/// AccountWorkers Information, Storage map for associating an account ID with a worker ID. If no worker exists, the query returns None.
	/// Keeps track of workerIds per account if any
	#[pallet::storage]
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

			match worker_keys {
				Some(keys) => {
					for id in 0..=keys {
						// Get the Worker associated with the creator and worker_id
						if let Some(worker) = WorkerClusters::<T>::get((creator.clone(), id)) {
							// Check if the API matches and throw an error if it does
							ensure!(api != worker.api, Error::<T>::WorkerExists);
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
				reputation: 0,
				start_block: blocknumber.clone(),
				status: WorkerStatusType::Inactive,
				status_last_updated: blocknumber.clone(),
				api: api,
				last_status_check: timestamp::Pallet::<T>::get(),
			};

			// update storage
			AccountWorkers::<T>::insert(creator.clone(), worker_id.clone());
			WorkerClusters::<T>::insert((creator.clone(), worker_id.clone()), worker.clone());

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
		pub fn remove_worker(origin: OriginFor<T>, worker_id: WorkerId) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			ensure!(
				WorkerClusters::<T>::get((creator.clone(), worker_id)) != None,
				Error::<T>::WorkerDoesNotExist
			);

			// update storage
			WorkerClusters::<T>::remove((creator.clone(), worker_id));

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
			worker_id: WorkerId,
			visibility: bool,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			let mut worker = WorkerClusters::<T>::get((creator.clone(), worker_id))
				.ok_or(Error::<T>::WorkerDoesNotExist)?;

			worker.status = if visibility {
				WorkerStatusType::Active
			} else {
				WorkerStatusType::Inactive
			};

			worker.last_status_check = timestamp::Pallet::<T>::get();

			WorkerClusters::<T>::insert((creator.clone(), worker_id), worker.clone());

			Self::deposit_event(Event::WorkerStatusUpdated {
				creator,
				worker_id,
				worker_status: worker.status,
			});

			Ok(().into())
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
	}

	impl<T: Config + timestamp::Config>
		WorkerInfoHandler<T::AccountId, WorkerId, BlockNumberFor<T>, T::Moment> for Pallet<T>
	{
		// Implementation of the WorkerInfoHandler trait, which provides methods for accessing worker cluster information.
		fn get_worker_cluster(
			worker_key: &(T::AccountId, WorkerId),
		) -> Option<Worker<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			WorkerClusters::<T>::get(worker_key)
		}

		// Implementation of the WorkerInfoHandler trait, which provides methods for updating worker cluster information.
		fn update_worker_cluster(
			worker_key: &(T::AccountId, WorkerId),
			worker: Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
		) {
			WorkerClusters::<T>::insert(worker_key, worker);
		}
	}
}
