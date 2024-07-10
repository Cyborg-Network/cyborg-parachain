#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;

pub use types::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::{WeightInfo, pallet_prelude::*};
	use scale_info::prelude::vec::Vec;
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
    pub fn WorkerCountDefault() -> WorkerId {
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
	pub type WorkerClusters<T: Config> = 
		StorageMap<_, Twox64Concat, (T::AccountId, WorkerId), Worker<T::AccountId, BlockNumberFor<T>>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		WorkerRegistered{ creator: T::AccountId },
		WorkerRemoved{ creator: T::AccountId, worker_id: WorkerId },
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
		// #[pallet::weight(T::WeightInfo::register_worker())]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn register_worker(
			origin: OriginFor<T>, 
			ip: Option<Ip>, 
			domain: Option<Domain>,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			// check ip or domain exists
			ensure!(ip.clone().and_then(|ip| ip.ipv4).is_some() || ip.clone().and_then(|ip| ip.ipv6).is_some() || domain.is_some(), Error::<T>::WorkerRegisterMissingIpOrDomain);

			let api = WorkerAPI {
				ip, domain
			};
			let worker_keys = AccountWorkers::<T>::get(creator.clone());

			match worker_keys {
				Some(keys) => {
			        for id in 0..=keys {
						// Get the Worker associated with the creator and worker_id
						if let Some(worker) = WorkerClusters::<T>::get((creator.clone(), id)) {
							// Check if the API matches and throw an error if it does
							ensure!(api != worker.api, Error::<T>::WorkerExists);
						}
					}
				},
				None => {}
			}

			let worker_id: WorkerId = match AccountWorkers::<T>::get(creator.clone()) {
				Some(id) => {
					AccountWorkers::<T>::insert(creator.clone(), id + 1);
					id + 1
				},
				None => {
					AccountWorkers::<T>::insert(creator.clone(), 0);
					0
				}
			};

			let worker = Worker {
				id: worker_id.clone(),
				owner: creator.clone(),
				start_block: <frame_system::Pallet<T>>::block_number(),
				status: WorkerStatusType::Inactive,
				api: api,
			};

			// update storage
			AccountWorkers::<T>::insert(creator.clone(), worker_id.clone());
			WorkerClusters::<T>::insert((creator.clone(), worker_id.clone()), worker);

			// Emit an event.
			Self::deposit_event(Event::WorkerRegistered { creator });

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// Remove Worker from storage
		#[pallet::call_index(1)]
		// #[pallet::weight(T::WeightInfo::remove_worker())]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]		
		pub fn remove_worker(
			origin: OriginFor<T>, 
			worker_id: WorkerId, 
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			
			ensure!(WorkerClusters::<T>::get((creator.clone(), worker_id)) != None, 
			Error::<T>::WorkerDoesNotExist);

			// update storage
			WorkerClusters::<T>::remove((creator.clone(), worker_id));

			// Emit an event.
			Self::deposit_event(Event::WorkerRemoved { creator, worker_id });


			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_active_workers() -> Option<Vec<((T::AccountId, WorkerId),Worker<T::AccountId, BlockNumberFor<T>>)>> {
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
