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

	/// Storage map to index workers by their geohash at a specific precision level
	#[pallet::storage]
	pub type WorkersByGeohash<T: Config> = StorageMap<
		_,
		Twox64Concat,
		GeoHash,                                              // Geohash at specified precision
		BoundedVec<(T::AccountId, WorkerId), ConstU32<1000>>, // Increased limit for geohash areas
		ValueQuery,
	>;

	/// Storage map to track all geohash precision levels used in the system
	#[pallet::storage]
	pub type GeohashPrecisionLevels<T: Config> =
		StorageValue<_, BoundedVec<u8, ConstU32<12>>, ValueQuery>;

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
			let geohash_precision = 6; // ~1.2km precision
			let geohash = Self::coordinates_to_geohash(latitude, longitude, geohash_precision);

			let worker_location = Location {
				coordinates: GeoCoordinates {
					latitude,
					longitude,
					geohash: geohash.clone(),
				},
				geohash_precision,
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

			WorkersByGeohash::<T>::try_append(geohash, (creator.clone(), worker_id.clone()))
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
				// Remove from geohash index
				WorkersByGeohash::<T>::mutate(&worker.location.coordinates.geohash, |workers| {
					if let Some(pos) = workers
						.iter()
						.position(|w| w == &(creator.clone(), worker_id))
					{
						workers.swap_remove(pos);
					}
				});

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
		}
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

		/// Convert coordinates to geohash at specified precision
		pub fn coordinates_to_geohash(
			latitude: Latitude,
			longitude: Longitude,
			precision: u8,
		) -> GeoHash {
			// Normalize coordinates to 0-180 range
			let lat_normalized = (latitude + 90) as u32;
			let lon_normalized = (longitude + 180) as u32;

			// Simple bit interleaving
			let mut hash = Vec::new();
			for i in 0..precision {
				let lat_bit = (lat_normalized >> (i % 5)) & 1;
				let lon_bit = (lon_normalized >> (i % 5)) & 1;
				hash.push(b'0' + lat_bit as u8);
				hash.push(b'0' + lon_bit as u8);
			}

			BoundedVec::try_from(hash).expect("Geohash length exceeds maximum")
		}

		/// Get geohash neighbors (adjacent geohash areas)
		fn get_geohash_neighbors(geohash: &GeoHash) -> Vec<GeoHash> {
			vec![geohash.clone()]
		}

		/// Update worker location and geohash indices
		#[allow(dead_code)]
		fn update_worker_location(
			worker_key: &(T::AccountId, WorkerId),
			worker_type: WorkerType,
			new_latitude: Latitude,
			new_longitude: Longitude,
		) -> DispatchResult {
			// Get the worker
			let mut worker = match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::get(worker_key),
				WorkerType::Executable => ExecutableWorkers::<T>::get(worker_key),
			}
			.ok_or(Error::<T>::WorkerDoesNotExist)?;

			// Remove from old geohash index
			WorkersByGeohash::<T>::mutate(&worker.location.coordinates.geohash, |workers| {
				if let Some(pos) = workers.iter().position(|w| w == worker_key) {
					workers.swap_remove(pos);
				}
			});

			// Calculate new geohash
			let precision = worker.location.geohash_precision;
			let new_geohash = Self::coordinates_to_geohash(new_latitude, new_longitude, precision);

			// Update worker location
			worker.location.coordinates.latitude = new_latitude;
			worker.location.coordinates.longitude = new_longitude;
			worker.location.coordinates.geohash = new_geohash.clone();

			// Add to new geohash index
			WorkersByGeohash::<T>::try_append(new_geohash, worker_key.clone())
				.map_err(|_| Error::<T>::TooManyWorkersAtLocation)?;

			// Update worker in storage
			match worker_type {
				WorkerType::Docker => WorkerClusters::<T>::insert(worker_key, worker),
				WorkerType::Executable => ExecutableWorkers::<T>::insert(worker_key, worker),
			}

			Ok(())
		}

		/// Get workers in the same geohash area
		pub fn get_workers_in_geohash_area(
			geohash: GeoHash,
		) -> BoundedVec<(T::AccountId, WorkerId), ConstU32<1000>> {
			WorkersByGeohash::<T>::get(geohash)
		}

		/// Get workers near a location (checks neighboring geohashes)
		pub fn get_workers_near_location(
			latitude: Latitude,
			longitude: Longitude,
			precision: u8,
		) -> Vec<Worker<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			let geohash = Self::coordinates_to_geohash(latitude, longitude, precision);
			let neighbors = Self::get_geohash_neighbors(&geohash);

			let mut workers = Vec::new();

			for neighbor in neighbors {
				for (owner, id) in WorkersByGeohash::<T>::get(neighbor).into_inner() {
					if let Some(worker) = WorkerClusters::<T>::get((owner.clone(), id))
						.or_else(|| ExecutableWorkers::<T>::get((owner, id)))
					{
						workers.push(worker);
					}
				}
			}

			workers
		}

		/// Find closest workers to a location within radius (km)
		pub fn find_workers_in_radius(
			latitude: Latitude,
			longitude: Longitude,
			radius_km: u32,
		) -> Vec<Worker<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			let mut precision = 6; // ~1.2km
			if radius_km > 20 {
				precision = 5; // ~5km
			}
			if radius_km > 80 {
				precision = 4; // ~20km
			}

			let mut workers = Self::get_workers_near_location(latitude, longitude, precision);

			if precision > 4 {
				workers.retain(|worker| {
					let distance = Self::calculate_distance(
						latitude,
						longitude,
						worker.location.coordinates.latitude,
						worker.location.coordinates.longitude,
					);
					distance <= radius_km as f64
				});
			}

			workers
		}

		/// Calculate distance between two points in kilometers (Haversine formula)
		fn calculate_distance(lat1: Latitude, lon1: Longitude, lat2: Latitude, lon2: Longitude) -> f64 {
			use core::f64::consts::PI;

			let lat1_rad = lat1 as f64 * PI / 180.0;
			let lon1_rad = lon1 as f64 * PI / 180.0;
			let lat2_rad = lat2 as f64 * PI / 180.0;
			let lon2_rad = lon2 as f64 * PI / 180.0;

			let dlat = lat2_rad - lat1_rad;
			let dlon = lon2_rad - lon1_rad;

			let a = libm::pow(libm::sin(dlat / 2.0), 2.0)
				+ libm::cos(lat1_rad) * libm::cos(lat2_rad) * libm::pow(libm::sin(dlon / 2.0), 2.0);
			let c = 2.0 * libm::atan2(libm::sqrt(a), libm::sqrt(1.0 - a));

			6371.0 * c // Earth radius in km
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
