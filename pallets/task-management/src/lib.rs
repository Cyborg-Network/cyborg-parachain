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

use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use scale_info::prelude::vec::Vec;
use sp_core::hash::H256;

pub type TaskId = u64;

pub use cyborg_primitives::task::*;
use cyborg_primitives::worker::{WorkerId, WorkerStatusType};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_edge_connect::{AccountWorkers, WorkerClusters};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_edge_connect::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn task_status)]
	pub type TaskStatus<T: Config> = StorageMap<_, Twox64Concat, TaskId, TaskStatusType, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn task_allocations)]
	pub type TaskAllocations<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, (T::AccountId, WorkerId), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn task_owners)]
	pub type TaskOwners<T: Config> = StorageMap<_, Twox64Concat, TaskId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_task_id)]
	pub type NextTaskId<T: Config> = StorageValue<_, TaskId, ValueQuery>;

	/// Task Information
	#[pallet::storage]
	#[pallet::getter(fn get_tasks)]
	pub type Tasks<T: Config> =
		StorageMap<_, Identity, TaskId, TaskInfo<T::AccountId, BlockNumberFor<T>>, OptionQuery>;

	/// Private task verifications
	#[pallet::storage]
	#[pallet::getter(fn task_verifications)]
	type TaskVerifications<T: Config> =
		StorageMap<_, Blake2_128Concat, TaskId, Verifications<T::AccountId>, OptionQuery>;

	/// Pallets use events to inform users when important changes are made.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TaskScheduled {
			assigned_worker: (T::AccountId, WorkerId),
			task_owner: T::AccountId,
			task_id: TaskId,
			task: BoundedVec<u8, ConstU32<128>>,
		},
		SubmittedCompletedTask {
			task_id: TaskId,
			assigned_verifier: (T::AccountId, WorkerId),
		},
		VerifierResolverAssigned {
			task_id: TaskId,
			assigned_resolver: (T::AccountId, WorkerId),
		},
		VerifiedCompletedTask {
			task_id: TaskId,
		},
		ResolvedCompletedTask {
			task_id: TaskId,
		},
		TaskReassigned {
			task_id: TaskId,
			assigned_executor: (T::AccountId, WorkerId),
		},
	}

	/// Errors inform users that something went wrong.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::error]
	pub enum Error<T> {
		UnassignedTaskId,
		InvalidTaskOwner,
		RequireAssignedTask,
		RequireAssignedVerifier,
		RequireAssignedVerifierCompletedHash,
		RequireAssignedResolver,
		NoWorkersAvailable,
		TaskVerificationNotFound,
		NoNewWorkersAvailable,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Creates a task and assigns it to an available worker
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::task_scheduler(task_data.len() as u32))]
		pub fn task_scheduler(
			origin: OriginFor<T>,
			task_data: BoundedVec<u8, ConstU32<128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let existing_workers = AccountWorkers::<T>::iter().next().is_some();
			ensure!(existing_workers, Error::<T>::NoWorkersAvailable);

			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.wrapping_add(1));

			// Select one worker randomly.
			let workers: Vec<_> = WorkerClusters::<T>::iter().collect::<Vec<_>>(); // TODO: Update for only active workers in production
			let random_index = (sp_io::hashing::blake2_256(&task_data)[0] as usize) % workers.len();
			let selected_worker: (T::AccountId, WorkerId) = workers[random_index].0.clone();

			let task_info = TaskInfo {
				task_owner: who.clone(),
				create_block: <frame_system::Pallet<T>>::block_number(),
				metadata: task_data.clone(),
				time_elapsed: None,
				average_cpu_percentage_use: None,
				task_type: TaskType::Docker,
				result: None,
			};

			// Assign task to worker and set task owner.
			TaskAllocations::<T>::insert(task_id, selected_worker.clone());
			TaskOwners::<T>::insert(task_id, who.clone());
			Tasks::<T>::insert(task_id, task_info);
			TaskStatus::<T>::insert(task_id, TaskStatusType::Assigned);

			// Emit an event.
			Self::deposit_event(Event::TaskScheduled {
				assigned_worker: selected_worker,
				task_owner: who,
				task_id,
				task: task_data,
			});
			Ok(())
		}

		//// Assignee submits completed task for verification and validation from other workers
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::submit_completed_task(u32::MAX))]
		pub fn submit_completed_task(
			origin: OriginFor<T>,
			task_id: TaskId,
			completed_hash: H256,
			result: BoundedVec<u8, ConstU32<128>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let task_assignee = TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
			ensure!(task_assignee.0 == who, Error::<T>::InvalidTaskOwner);
			ensure!(
				TaskStatus::<T>::get(task_id) == Some(TaskStatusType::Assigned),
				Error::<T>::RequireAssignedTask
			);

			// update storage info
			TaskStatus::<T>::insert(task_id, TaskStatusType::PendingValidation);

			let mut ver = Verifications {
				executor: VerificationHashes {
					account: who.clone(),
					completed_hash: Some(completed_hash),
				},
				verifier: None,
				resolver: None,
			};

			let workers: Vec<_> = WorkerClusters::<T>::iter()
				.filter(|&(_, ref worker)| {
					worker.status == WorkerStatusType::Inactive && worker.owner != who.clone()
				}) // TODO: change Inactive to Active with oracle
				.collect::<Vec<_>>();

			ensure!(workers.len() > 0, Error::<T>::NoNewWorkersAvailable);

			let random_index = (sp_io::hashing::blake2_256(&ver.encode())[0] as usize) % workers.len();
			let assigned_verifier: (T::AccountId, WorkerId) = workers[random_index].0.clone();

			ver.verifier = Some(VerificationHashes {
				account: assigned_verifier.0.clone(),
				completed_hash: None,
			});

			Tasks::<T>::mutate(task_id, |task| {
				if let Some(ref mut raw_task) = task {
					raw_task.result = Some(result);
				}
			});

			TaskVerifications::<T>::insert(task_id, ver.clone());
			// Emit an event.
			Self::deposit_event(Event::SubmittedCompletedTask {
				task_id,
				assigned_verifier,
			});
			Ok(())
		}

		/// Verifies completed task once an assignedverifier have succussfully validate correct completed task
		/// Can only be called from root
		/// Assign new verifier as resolver if verification fails. Resolver will determine correct result.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::verify_completed_task(u32::MAX))]
		pub fn verify_completed_task(
			origin: OriginFor<T>,
			// verifier_account: T::AccountId, // Change if using oracle
			task_id: TaskId,
			completed_hash: H256,
		) -> DispatchResult {
			// ensure_root(origin)?;
			let verifier_account = ensure_signed(origin)?;
			ensure!(
				TaskStatus::<T>::get(task_id) == Some(TaskStatusType::PendingValidation),
				Error::<T>::RequireAssignedTask
			);
			let task_verification = TaskVerifications::<T>::get(task_id);

			// check completed hashes are the same
			match task_verification {
				Some(ref verification) => {
					ensure!(
						verification
							.verifier
							.as_ref()
							.map_or(false, |v| v.account == verifier_account),
						Error::<T>::RequireAssignedVerifier
					);
					// Hashes Match
					if verification.executor.completed_hash == Some(completed_hash) {
						TaskStatus::<T>::insert(task_id, TaskStatusType::Completed);
						// Emit an event.
						Self::deposit_event(Event::VerifiedCompletedTask { task_id });
					} else {
						// Assign new verifier as resolver if verification does not match
						let mut new_verification = verification.clone();

						// Find available workers that are not the executor or current verifier (TODO: may be redesigned into a hook)
						let workers: Vec<_> = WorkerClusters::<T>::iter()
							.filter(|&(_, ref worker)| {
								worker.status == WorkerStatusType::Inactive // TODO: change Inactive to Active with oracle 
							&& verification.verifier.as_ref().map_or(false, |v| v.account != worker.owner)
							&& worker.owner != verification.executor.account.clone()
							})
							.collect::<Vec<_>>();

						ensure!(workers.len() > 0, Error::<T>::NoNewWorkersAvailable);

						let mut task_verification_encoded = task_verification.encode();
						let block_number_encoded = <frame_system::Pallet<T>>::block_number().encode();
						task_verification_encoded.extend(block_number_encoded);

						let random_index =
							(sp_io::hashing::blake2_256(&task_verification_encoded)[0] as usize) % workers.len();
						let assigned_resolver: (T::AccountId, WorkerId) = workers[random_index].0.clone();

						new_verification.verifier = Some(VerificationHashes {
							account: verifier_account.clone(),
							completed_hash: Some(completed_hash),
						});
						new_verification.resolver = Some(VerificationHashes {
							account: assigned_resolver.0.clone(),
							completed_hash: None,
						});
						TaskVerifications::<T>::insert(task_id, new_verification);

						// Emit an event.
						Self::deposit_event(Event::VerifierResolverAssigned {
							task_id,
							assigned_resolver,
						});
					};
				}
				None => {
					return Err(Error::<T>::TaskVerificationNotFound.into());
				}
			};
			Ok(())
		}

		/// Checks whether resolver matches the executor or verifier.
		/// If it matches one, the task is resolved and award is split between the matching pair. The failing worker is slashed.
		/// If no matches, the task is reassigned to a new executor and cycle repeats
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::resolve_completed_task(u32::MAX))]
		pub fn resolve_completed_task(
			origin: OriginFor<T>,
			// resolver_account: T::AccountId,
			task_id: TaskId,
			completed_hash: H256,
		) -> DispatchResult {
			// ensure_root(origin)?;
			let resolver_account = ensure_signed(origin)?;
			ensure!(
				TaskStatus::<T>::get(task_id) == Some(TaskStatusType::PendingValidation),
				Error::<T>::RequireAssignedTask
			);
			let task_verification = TaskVerifications::<T>::get(task_id);

			// check completed hashes are the same
			match task_verification {
				Some(ref verification) => {
					let Verifications {
						ref executor,
						ref verifier,
						ref resolver,
					} = verification;

					ensure!(
						verifier
							.as_ref()
							.map_or(false, |v| v.completed_hash.is_some()),
						Error::<T>::RequireAssignedVerifierCompletedHash
					);
					ensure!(
						resolver
							.as_ref()
							.map_or(false, |v| v.account == resolver_account),
						Error::<T>::RequireAssignedResolver
					);

					if executor.completed_hash == Some(completed_hash)
						|| verifier
							.as_ref()
							.map_or(false, |v| v.completed_hash == Some(completed_hash))
					{
						TaskStatus::<T>::insert(task_id, TaskStatusType::Completed);
						// Emit an event.
						Self::deposit_event(Event::ResolvedCompletedTask { task_id });
					// TODO! Reward and Slash disputing completed_hash (require implement tokenomics)
					} else {
						// reassign task to new executor
						// reassign task to T::AccountId that is neither of the current executor or verifier or resolver for the next cycle
						let workers: Vec<_> = WorkerClusters::<T>::iter()
							.filter(|&(_, ref worker)| {
								worker.status == WorkerStatusType::Inactive // change Inactive to Active with oracle 
							&& worker.owner != resolver_account
							&& verifier.as_ref().map_or(false, |v| v.account != worker.owner)
							&& worker.owner != executor.account.clone()
							})
							.collect::<Vec<_>>();

						ensure!(workers.len() > 0, Error::<T>::NoNewWorkersAvailable);

						let mut task_verification_encoded = task_verification.encode();
						let block_number_encoded = <frame_system::Pallet<T>>::block_number().encode();
						task_verification_encoded.extend(block_number_encoded);

						let random_index =
							(sp_io::hashing::blake2_256(&task_verification_encoded)[0] as usize) % workers.len();
						let assigned_new_executor: (T::AccountId, WorkerId) = workers[random_index].0.clone();

						TaskVerifications::<T>::remove(task_id);
						TaskStatus::<T>::insert(task_id, TaskStatusType::Assigned);
						TaskAllocations::<T>::insert(task_id, assigned_new_executor.clone());
						// Emit an event.
						Self::deposit_event(Event::TaskReassigned {
							task_id,
							assigned_executor: assigned_new_executor,
						});
					}
				}
				None => {
					return Err(Error::<T>::TaskVerificationNotFound.into());
				}
			}
			Ok(())
		}
	}
}
