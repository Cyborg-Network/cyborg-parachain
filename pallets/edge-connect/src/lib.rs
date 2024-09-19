#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::{*, WeightInfo as EdgeConnectWeightInfo};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;

pub use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;
  use pallet_timestamp as timestamp;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// /// A type representing the weights required by the dispatchables of this pallet.
		type EdgeConnectWeightInfo: EdgeConnectWeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn WorkerCountDefault() -> WorkerId {
		0
	}

  #[pallet::type_value]
  pub fn WorkerReputationDefault() -> WorkerReputation {
    0
  }

	/// Keeps track of workerIds per account if any
	#[pallet::storage]
	#[pallet::getter(fn account_workers)]
	pub type AccountWorkers<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, WorkerId, OptionQuery>;

	/// Worker Cluster information
	#[pallet::storage]
	#[pallet::getter(fn get_worker_clusters)]
	pub type WorkerClusters<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(T::AccountId, WorkerId),
		Worker<T::AccountId, BlockNumberFor<T>, T::Moment>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		WorkerRegistered {
			creator: T::AccountId,
			worker: (T::AccountId, WorkerId),
			domain: Domain,
		},
		WorkerRemoved {
			creator: T::AccountId,
			worker_id: WorkerId,
		},
    WorkerStatusUpdated {
      creator: T::AccountId,   
      worker_id: WorkerId,
      worker_status: WorkerStatusType,
    },
	}

	/// Pallet Errors
	#[pallet::error]
	pub enum Error<T> {
		WorkerRegisterMissingIpOrDomain,
		WorkerExists,
		WorkerDoesNotExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Registers a Worker with either a domain or ip and initialize it with an inactive status.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::EdgeConnectWeightInfo::register_worker())]
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
      let worker_location = Location { latitude, longitude };
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

			let worker = Worker {
				id: worker_id.clone(),
				owner: creator.clone(),
        location: worker_location,
        specs: worker_specs,
        reputation: 0,
				start_block: <frame_system::Pallet<T>>::block_number(),
				status: WorkerStatusType::Inactive,
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

		/// Remove Worker from storage
		#[pallet::call_index(1)]
		#[pallet::weight(T::EdgeConnectWeightInfo::remove_worker())]
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

    #[pallet::call_index(2)]
    #[pallet::weight(T::EdgeConnectWeightInfo::toggle_worker_visibility())]
    pub fn toggle_worker_visibility(
      origin: OriginFor<T>, 
      worker_id: WorkerId, 
      visibility: bool
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

      Self::deposit_event(Event::WorkerStatusUpdated { creator, worker_id, worker_status: worker.status});

      Ok(().into())
    }
	}

	impl<T: Config> Pallet<T> {
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
}
