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

pub use cyborg_primitives::miner::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use cyborg_primitives::task::TaskId;
	use frame_support::sp_runtime::Saturating;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, BoundedVec};
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

	// A helper function providing a default value for miner IDs.
	// #[pallet::type_value]
	// pub fn MinerCountDefault() -> MinerId {
	// 	0
	// }

	// A helper function providing a default value for miner reputations.
	#[pallet::type_value]
	pub fn MinerReputationDefault<T: Config>() -> MinerReputation<BlockNumberFor<T>> {
		MinerReputation::default()
	}

	/// AccountMiners Information, Storage map for associating an account ID with a miner ID. If no miner exists, the query returns None.
	/// Keeps track of MinerIds per account if any
	#[pallet::storage]
	#[pallet::getter(fn account_miners)]
	pub type AccountMiners<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, MinerId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn suspended_miners)]
	pub type SuspendedMiners<T: Config> = StorageMap<
		_,
		Twox64Concat,
		MinerId,
		(BlockNumberFor<T>, SuspensionReason),
		OptionQuery,
	>;

	/// Cloud Miner information, Storage map to keep track of detailed miner information for each (account ID, miner ID) pair.
	#[pallet::storage]
	pub type CloudMiners<T: Config> = StorageMap<
		_,
		Twox64Concat,
		MinerId,
		Miner<T::AccountId, BlockNumberFor<T>, T::Moment>,
		OptionQuery,
	>;

	/// Edge Miner information, Storage map to keep track of detailed miner information for each (account ID, miner ID) pair.
	#[pallet::storage]
	pub type EdgeMiners<T: Config> = StorageMap<
		_,
		Twox64Concat,
		MinerId,
		Miner<T::AccountId, BlockNumberFor<T>, T::Moment>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn miners_under_maintenance)]
	pub type MinersUnderMaintenance<T:Config>=StorageMap<
		_,
		Twox64Concat,
		MinerId,
		BlockNumberFor<T>,
		OptionQuery
	>;

	/// The `Event` enum contains the various events that can be emitted by this pallet.
	/// Events are emitted when significant actions or state changes happen in the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a new miner is successfully registered.
		///
		/// - `creator`: The account ID of the miner's creator.
		/// - `miner`: A tuple containing the account ID of the miner owner and the miner ID.
		/// - `domain`: The domain associated with the
		MinerRegistered {
			creator: T::AccountId,
			miner: (T::AccountId, MinerId),
			domain: Domain,
		},

		/// Event emitted when a miner tries to re-register itself.
		///
		/// - `creator`: The account ID of the miner's creator.
		/// - `miner`: A tuple containing the account ID of the miner owner and the miner ID.
		MinerAlreadyRegistered {
			creator: T::AccountId,
			miner: (T::AccountId, MinerId),
			domain: Domain,
		},

		/// Event emitted when a miner is removed from the system.
		///
		/// - `creator`: The account ID of the miner's creator.
		/// - `miner_id`: The ID of the miner that was removed.
		MinerRemoved {
			creator: T::AccountId,
			miner_id: MinerId,
		},

		OracleStatusUpdated {
			worker: (T::AccountId, MinerId),
			online: bool,
		},
		OperationalStatusUpdated {
			worker: (T::AccountId, MinerId),
			status: OperationalStatus,
		},

		/// Event emitted when a miner is penalized
		MinerPenalized {
			miner: MinerId,
			penalty: i32,
			reason: PenaltyReason,
		},

		/// Event emitted when a miner is suspended
		MinerSuspended {
			miner:  MinerId,
			until_block: BlockNumberFor<T>,
		},

		/// Event emitted when a miner is put under review
		MinerUnderReview {
			miner: MinerId,
			reason: SuspensionReason,
		},

		/// Event emitted when a miner is banned
		MinerBanned {
			miner: MinerId,
			reason: SuspensionReason,
		},

		/// Event emitted when a miner is unsuspended
		MinerUnsuspended { miner: MinerId },

		/// Maintenance mode resolved by root/admin, miner restored to Available.
		MaintenanceResolved{miner:MinerId},

		/// Miner has been put under maintenance mode by its owner.
		MinerUnderMaintenance {
        miner: MinerId,
        who: T::AccountId,
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
		/// Error indicating that either the IP address or the domain was missing when attempting to register a miner.
		MinerRegisterMissingIpOrDomain,
		/// Error indicating that the miner already exists and cannot be registered again.
		MinerExists,
		/// Error indicating that the miner does not exist in the system when trying to perform actions (e.g., removal or status update).
		MinerDoesNotExist,
		/// Miner is suspended and cannot perform actions.
		MinerSuspended,
		/// Miner reputation is too low
		InsufficientReputation,
		/// Miner is busy
		MinerIsBusy,
		/// Miner is inactive
		MinerIsInactive,
		NotAuthorized,
		// Provided UUID exceeded MaxUuidLen
		UuidTooLong, 
		//When Miner is not Under Maintenance
		NotUnderMaintenance
	}

	// This block defines the dispatchable functions (calls) for the pallet.
	// Dispatchable functions are the publicly accessible functions that users or other pallets
	// can call to interact with the pallet. Each function has a weight and requires the user
	// to sign the transaction unless specified otherwise.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers a Miner with either a domain and initialize it with an inactive status.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_miner())]
		pub fn register_miner(
			origin: OriginFor<T>,
			miner_type: MinerType,
			miner_uuid: MinerId,
			domain: Domain,
			latitude: Latitude,
			longitude: Longitude,
			ram: RamBytes,
			storage: StorageBytes,
			cpu: CpuCores,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			let api = MinerAPI { domain };
			let miner_location = Location {
				latitude,
				longitude,
			};
			let miner_specs = MinerSpecs { ram, storage, cpu };

				//  Check if the miner already exists
			let miner_exists = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::contains_key(miner_uuid.clone()),
				MinerType::Edge => EdgeMiners::<T>::contains_key(miner_uuid.clone()),
			};

			if miner_exists {
				// Emit an event for re-registration attempt
				let existing_miner = match miner_type {
					MinerType::Cloud => CloudMiners::<T>::get(&miner_uuid),
					MinerType::Edge => EdgeMiners::<T>::get(&miner_uuid),
				};

				if let Some(miner) = existing_miner {
					Self::deposit_event(Event::MinerAlreadyRegistered {
						creator: creator.clone(),
						miner: (miner.owner.clone(), miner.id.clone()),
						domain: miner.api.domain.clone(),
					});
				}
				return Err(Error::<T>::MinerExists.into());
			}

let blocknumber = <frame_system::Pallet<T>>::block_number();
			let miner = Miner {
				id: miner_uuid.clone(),
				owner: creator.clone(),
				location: miner_location,
				specs: miner_specs,
				reputation: MinerReputation::<BlockNumberFor<T>>::default(),
				current_task: None,
				start_block: blocknumber.clone(),
				oracle_status: OracleStatus::Offline,
				operational_status: OperationalStatus::Available,
				status_last_updated: blocknumber.clone(),
				api: api,
				last_status_check: timestamp::Pallet::<T>::get(),
			};

			AccountMiners::<T>::insert(creator.clone(), miner_uuid.clone());


			//  Store miner efficiently
				match miner_type {
					MinerType::Cloud => CloudMiners::<T>::insert(&miner_uuid, miner.clone()),
					MinerType::Edge => EdgeMiners::<T>::insert(&miner_uuid, miner.clone()),
				}


			
			// Emit an event.
			Self::deposit_event(Event::MinerRegistered {
				creator: creator.clone(),
				miner: (miner.owner.clone(), miner.id.clone()),
				domain: miner.api.domain.clone(),
			});

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// Remove a miner from storage an deactivates it
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_miner())]
		pub fn remove_miner(
			origin: OriginFor<T>,
			miner_type: MinerType,
			miner_id: MinerId,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			match miner_type {
				MinerType::Cloud => {
					ensure!(
						CloudMiners::<T>::get(&miner_id) != None,
						Error::<T>::MinerDoesNotExist
					);

					// update storage
					CloudMiners::<T>::remove(&miner_id);
				}
				MinerType::Edge => {
					ensure!(
						EdgeMiners::<T>::get(&miner_id) != None,
						Error::<T>::MinerDoesNotExist
					);

					// update storage
					EdgeMiners::<T>::remove(&miner_id);
				}
			}

			// Emit an event.
			Self::deposit_event(Event::MinerRemoved { creator, miner_id });

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}
 
		/// Updates the oracle status (callable by oracle feeder)
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::update_oracle_status())]
		pub fn update_oracle_status(
			origin: OriginFor<T>,
			miner_owner: T::AccountId,
			miner_id: MinerId,
			miner_type: MinerType,
			online: bool,
		) -> DispatchResult {
			ensure_signed(origin)?;

			match miner_type {
				MinerType::Cloud => {
					CloudMiners::<T>::mutate(miner_id.clone(), |miner_option| {
						if let Some(miner) = miner_option {
							miner.oracle_status = if online {
								OracleStatus::Online
							} else {
								OracleStatus::Offline
							};
							miner.last_status_check = timestamp::Pallet::<T>::get();
							Ok(())
						} else {
							Err(Error::<T>::MinerDoesNotExist)
						}
					})
				}
				MinerType::Edge => {
					EdgeMiners::<T>::mutate(miner_id.clone(), |miner_option| {
						if let Some(miner) = miner_option {
							miner.oracle_status = if online {
								OracleStatus::Online
							} else {
								OracleStatus::Offline
							};
							miner.last_status_check = timestamp::Pallet::<T>::get();
							Ok(())
						} else {
							Err(Error::<T>::MinerDoesNotExist)
						}
					})
				}
			}?;

			Self::deposit_event(Event::OracleStatusUpdated {
				worker: (miner_owner, miner_id),
				online,
			});

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::penalize_miner())]
		pub fn penalize_miner(
			origin: OriginFor<T>,
			miner_id: MinerId,
			miner_type: MinerType,
			penalty: i32,
			reason: PenaltyReason,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::apply_penalty(&miner_id, &miner_type, penalty, reason)?;

			Ok(())
		}

		/// Manually suspend a miner (root only)
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::suspend_miner())]
		pub fn suspend_miner(
			origin: OriginFor<T>,
			miner_id: MinerId,
			miner_type: MinerType,
			blocks: BlockNumberFor<T>,
			reason: SuspensionReason,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::suspend_miners(&miner_id, &miner_type, blocks, reason)
		}

		/// Manually ban a miner (root only)
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::ban_miner())]
		pub fn ban_miner(
			origin: OriginFor<T>,
			miner_id: MinerId,
			miner_type: MinerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::ban_miners(&miner_id, miner_type, reason)
		}

		/// Lift suspension from a miner (root only)
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unsuspend_miner())]
		pub fn unsuspend_miner(
			origin: OriginFor<T>,
			miner_id: MinerId,
			miner_type: MinerType,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::lift_suspension(&miner_id, &miner_type)
		}

		/// Updates the operational status (callable by miner itself)
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::update_operational_status())]
		pub fn update_operational_status(
			origin: OriginFor<T>,
			miner_type: MinerType,
			miner_id: MinerId,
			status: OperationalStatus,
		) -> DispatchResult {
			let creator = ensure_signed(origin)?;
			let status_clone = status.clone();

			match miner_type {
				MinerType::Cloud => {
					CloudMiners::<T>::mutate(miner_id.clone(), |miner_option| {
						if let Some(miner) = miner_option {
							// Miners can only set Available or Busy status
							if matches!(status, OperationalStatus::Suspended) {
								return Err(Error::<T>::NotAuthorized.into());
							}
							miner.operational_status = status;
							miner.status_last_updated = <frame_system::Pallet<T>>::block_number();
							Ok(())
						} else {
							Err(Error::<T>::MinerDoesNotExist)
						}
					})
				}
				MinerType::Edge => {
					EdgeMiners::<T>::mutate(miner_id.clone(), |miner_option| {
						if let Some(miner) = miner_option {
							// Miners can only set Available or Busy status
							if matches!(status, OperationalStatus::Suspended) {
								return Err(Error::<T>::NotAuthorized.into());
							}
							miner.operational_status = status;
							miner.status_last_updated = <frame_system::Pallet<T>>::block_number();
							Ok(())
						} else {
							Err(Error::<T>::MinerDoesNotExist)
						}
					})
				}
			}?;

			Self::deposit_event(Event::OperationalStatusUpdated {
				worker: (creator, miner_id),
				status: status_clone,
			});

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::request_maintenance())]
		pub fn request_maintenance(
			origin: OriginFor<T>,
			miner_id: MinerId,
			miner_type: MinerType,
		) -> DispatchResult {
			let is_root = ensure_root(origin.clone()).is_ok();

			let who = if is_root {
				// Root caller: fetch miner owner
				let miner = Self::get_miner(&miner_id, &miner_type)
					.ok_or(Error::<T>::MinerDoesNotExist)?;
				miner.owner.clone()
			} else {
				// Normal user caller: ensure signed and get account id
				ensure_signed(origin.clone())?
			};
			

			let mut miner = Self::get_miner(&miner_id, &miner_type)
				.ok_or(Error::<T>::MinerDoesNotExist)?;

			// Ensure the miner belongs to the caller (if not root)
			if !is_root {
				ensure!(miner.owner == who, Error::<T>::NotAuthorized);
			}

			// Ensure the miner is in use or busy 
			ensure!(
				miner.operational_status == OperationalStatus::Busy,
				Error::<T>::MinerIsInactive
			);
			miner.operational_status = OperationalStatus::Maintenance; 
			miner.status_last_updated = <frame_system::Pallet<T>>::block_number();

			Self::update_miner(&miner_id, &miner_type, miner);

			// Record maintenance 
			MinersUnderMaintenance::<T>::insert(
				&miner_id, 
				<frame_system::Pallet<T>>::block_number(), 
			);

			Self::deposit_event(Event::MinerUnderMaintenance {
				miner: miner_id.clone(),
				who,
			});

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::resolve_maintenance())]
		pub fn resolve_maintenance(
			origin: OriginFor<T>,
			miner_id: MinerId,
			miner_type: MinerType,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Fetch miner
			let mut miner = Self::get_miner(&miner_id, &miner_type)
				.ok_or(Error::<T>::MinerDoesNotExist)?;

			ensure!(
				miner.operational_status == OperationalStatus::Maintenance,
				Error::<T>::NotUnderMaintenance
			);

			miner.operational_status = OperationalStatus::Available;
			miner.status_last_updated = <frame_system::Pallet<T>>::block_number();

			Self::update_miner(&miner_id, &miner_type, miner);

			// Remove from maintenance map
			MinersUnderMaintenance::<T>::remove(&miner_id);

			 Self::deposit_event(Event::MaintenanceResolved {
					miner: miner_id.clone(),
				});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Helper Function to retrieve all active miners from storage.
		// Filters miners based on their status (active or inactive).
		pub fn get_active_miners() -> Option<
			Vec<(
				MinerId,
				Miner<T::AccountId, BlockNumberFor<T>, T::Moment>,
			)>,
		> {
			let miners = CloudMiners::<T>::iter()
				.filter(|&(_, ref miner)| miner.can_accept_tasks())
				.collect::<Vec<_>>();

			if miners.is_empty() {
				None
			} else {
				Some(miners)
			}
		}

		pub fn is_registered_miner(account: &T::AccountId) -> bool {
			AccountMiners::<T>::contains_key(account)
		}

		/// Apply penalty to a miner's reputation
		pub fn apply_penalty(
			miner_key: &MinerId,
			miner_type: &MinerType,
			penalty: i32,
			reason: PenaltyReason,
		) -> DispatchResult {
			let mut miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_key),
				MinerType::Edge => EdgeMiners::<T>::get(miner_key),
			}
			.ok_or(Error::<T>::MinerDoesNotExist)?;

			// Apply penalty
			miner.reputation.score = miner.reputation.score.saturating_sub(penalty);
			miner.reputation.violations += 1;
			miner.reputation.last_updated = Some(<frame_system::Pallet<T>>::block_number());

			// Automatic suspension triggers
			if miner.reputation.score < 30 {
				// Severe penalty - suspend for 1000 blocks (~4 hours at 6s/block)
				Self::suspend_miners(
					miner_key,
					&miner_type.clone(),
					1000u32.into(),
					SuspensionReason::ReputationThreshold,
				)?;
			} else if miner.reputation.score < 50 {
				// Moderate penalty - put under review
				Self::put_miner_under_review(
					miner_key,
					&miner_type.clone(),
					SuspensionReason::ReputationThreshold,
				)?;
			} else if miner.reputation.violations > 10 {
				// Too many violations - review
				Self::put_miner_under_review(
					miner_key,
					&miner_type.clone(),
					SuspensionReason::RepeatedTaskFailures,
				)?;
			}

			// Update storage if not suspended
			if miner.operational_status != OperationalStatus::Suspended {
				match miner_type {
					MinerType::Cloud => CloudMiners::<T>::insert(miner_key, miner),
					MinerType::Edge => EdgeMiners::<T>::insert(miner_key, miner),
				}
			}

			Self::deposit_event(Event::MinerPenalized {
				miner: miner_key.clone(),
				penalty,
				reason,
			});

			Ok(())
		}

		pub fn get_miner(
			miner_key: &MinerId,
			miner_type: &MinerType,
		) -> Option<Miner<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			let miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_key),
				MinerType::Edge => EdgeMiners::<T>::get(miner_key),
			};

			miner
		}

		/// Check if miner can perform actions
		pub fn check_miner_status(
			miner_key: &MinerId,
			miner_type: &MinerType,
		) -> DispatchResult {
			let miner = Self::get_miner(miner_key, miner_type).ok_or(Error::<T>::MinerDoesNotExist)?;

			// Check if worker is suspended and if suspension period has expired
			if miner.is_suspended() {
				let current_block = <frame_system::Pallet<T>>::block_number();

				// If suspension period is over, auto-unsuspend
				if current_block >= miner.status_last_updated {
					let mut updated_miner = miner.clone();
					updated_miner.operational_status = OperationalStatus::Available;
					updated_miner.status_last_updated = current_block;

					// Update the miner status
					Self::update_miner(miner_key, miner_type, updated_miner);

					// Remove from suspended miner storage
					SuspendedMiners::<T>::remove(miner_key);
				} else {
					return Err(Error::<T>::MinerSuspended.into());
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

		pub fn update_miner_status(
			miner_id: &MinerId,
			miner_type: MinerType,
			new_status: bool,
		) -> DispatchResult {
			let mut miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_id),
				MinerType::Edge => EdgeMiners::<T>::get(miner_id),
			}
			.ok_or(Error::<T>::MinerDoesNotExist)?;

			miner.operational_status = if new_status {
				OperationalStatus::Available
			} else {
				OperationalStatus::Busy
			};
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::insert(miner_id, miner),
				MinerType::Edge => EdgeMiners::<T>::insert(miner_id, miner),
			}
			Ok(())
		}

		/// Updates the current task of the miner in question
		pub fn update_miner_current_task(
			miner_id: &MinerId,
			miner_type: &MinerType,
			current_task: Option<TaskId>,
		) -> DispatchResult {
			let mut miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_id),
				MinerType::Edge => EdgeMiners::<T>::get(miner_id),
			}
			.ok_or(Error::<T>::MinerDoesNotExist)?;

			miner.current_task = current_task;
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::insert(miner_id, miner),
				MinerType::Edge => EdgeMiners::<T>::insert(miner_id, miner),
			}
			Ok(())
		}

		/// Suspend a miner with a specific reason and duration
		pub fn suspend_miners(
			miner_key: &MinerId,
			miner_type: &MinerType,
			blocks: BlockNumberFor<T>,
			reason: SuspensionReason,
		) -> DispatchResult {
			let mut miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_key),
				MinerType::Edge => EdgeMiners::<T>::get(miner_key),
			}
			.ok_or(Error::<T>::MinerDoesNotExist)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let suspension_end = current_block.saturating_add(blocks);

			// Update miner status
			miner.operational_status = OperationalStatus::Suspended;
			miner.status_last_updated = suspension_end;
			miner.reputation.suspension_count += 1;

			// Update storage
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::insert(miner_key, miner),
				MinerType::Edge => EdgeMiners::<T>::insert(miner_key, miner),
			}

			// Record suspension
			SuspendedMiners::<T>::insert(miner_key, (suspension_end, reason.clone()));

			Self::deposit_event(Event::MinerSuspended {
				miner: miner_key.clone(),
				until_block: suspension_end,
			});

			Ok(())
		}

		/// Put miner under review
		fn put_miner_under_review(
			miner_key: &MinerId,
			miner_type: &MinerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			let mut miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_key),
				MinerType::Edge => EdgeMiners::<T>::get(miner_key),
			}
			.ok_or(Error::<T>::MinerDoesNotExist)?;

			miner.operational_status = OperationalStatus::Suspended; // Can't accept new tasks
			miner.reputation.review_count += 1;

			// Update storage
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::insert(miner_key, miner),
				MinerType::Edge => EdgeMiners::<T>::insert(miner_key, miner),
			}

			Self::deposit_event(Event::MinerUnderReview {
				miner: miner_key.clone(),
				reason,
			});

			Ok(())
		}

		/// Ban a miner permanently
		fn ban_miners(
			miner_key: &MinerId,
			miner_type: MinerType,
			reason: SuspensionReason,
		) -> DispatchResult {
			// Remove from active miners
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::remove(miner_key),
				MinerType::Edge => EdgeMiners::<T>::remove(miner_key),
			}

			Self::deposit_event(Event::MinerBanned {
				miner: miner_key.clone(),
				reason,
			});

			Ok(())
		}

		/// Lift suspension from a miner
		pub fn lift_suspension(
			miner_key: &MinerId,
			miner_type: &MinerType,
		) -> DispatchResult {
			let mut miner = match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_key),
				MinerType::Edge => EdgeMiners::<T>::get(miner_key),
			}
			.ok_or(Error::<T>::MinerDoesNotExist)?;

			// Only proceed if actually suspended
			if miner.operational_status != OperationalStatus::Suspended {
				return Ok(());
			}

			// Update miner status - set to Available when unsuspending
			miner.operational_status = OperationalStatus::Available;
			miner.status_last_updated = <frame_system::Pallet<T>>::block_number();

			// Update storage
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::insert(miner_key, miner),
				MinerType::Edge => EdgeMiners::<T>::insert(miner_key, miner),
			}

			// Remove from suspended miners
			SuspendedMiners::<T>::remove(miner_key);

			Self::deposit_event(Event::MinerUnsuspended {
				miner: miner_key.clone(),
			});

			Ok(())
		}
	}

	impl<T: Config + timestamp::Config>
		MinerInfoHandler<T::AccountId, MinerId, BlockNumberFor<T>, T::Moment> for Pallet<T>
	{
		// Implementation of the MinerInfoHandler trait, which provides methods for accessing miner cluster information.
		fn get_miner(
			miner_key: &MinerId,
			miner_type: &MinerType,
		) -> Option<Miner<T::AccountId, BlockNumberFor<T>, T::Moment>> {
			match miner_type {
				MinerType::Cloud => CloudMiners::<T>::get(miner_key),
				MinerType::Edge => EdgeMiners::<T>::get(miner_key),
			}
		}

		// Implementation of the MinerInfoHandler trait, which provides methods for updating miner cluster information.
		fn update_miner(
			miner_key: &MinerId,
			miner_type: &MinerType,
			miner: Miner<T::AccountId, BlockNumberFor<T>, T::Moment>,
		) {
			match miner_type {
				MinerType::Cloud => {
					CloudMiners::<T>::insert(miner_key, miner);
				}
				MinerType::Edge => {
					EdgeMiners::<T>::insert(miner_key, miner);
				}
			}
		}
	}
}
