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

use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use scale_info::prelude::vec::Vec;
use sp_core::hash::H256;

pub use cyborg_primitives::task::*;
use cyborg_primitives::worker::{WorkerId, WorkerStatusType};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_edge_connect::{AccountWorkers, ExecutableWorkers, WorkerClusters};
	use pallet_payment::ComputeHours;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_edge_connect::Config + pallet_payment::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Status of tasks within the system.
	#[pallet::storage]
	pub type TaskStatus<T: Config> = StorageMap<_, Twox64Concat, TaskId, TaskStatusType, OptionQuery>;

	/// Allocation of tasks to workers.
	#[pallet::storage]
	pub type TaskAllocations<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, (T::AccountId, WorkerId), OptionQuery>;

	/// Owners of the tasks.
	#[pallet::storage]
	pub type TaskOwners<T: Config> = StorageMap<_, Twox64Concat, TaskId, T::AccountId, OptionQuery>;

	/// The next task ID to be assigned.
	#[pallet::storage]
	pub type NextTaskId<T: Config> = StorageValue<_, TaskId, ValueQuery>;

	/// Task metadata and information.
	#[pallet::storage]
	pub type Tasks<T: Config> =
		StorageMap<_, Identity, TaskId, TaskInfo<T::AccountId, BlockNumberFor<T>>, OptionQuery>;

	/// Private task verifications
	#[pallet::storage]
	type TaskVerifications<T: Config> =
		StorageMap<_, Blake2_128Concat, TaskId, Verifications<T::AccountId>, OptionQuery>;

	/// Getter for TaskVerifications.
	impl<T: Config> Pallet<T> {
		/// Public getter for TaskVerifications.
		pub fn get_task_verifications(task_id: TaskId) -> Option<Verifications<T::AccountId>> {
			TaskVerifications::<T>::get(task_id)
		}
	}

	/// Pallets use events to inform users when important changes are made.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A task has been scheduled and assigned to a worker.
		TaskScheduled {
			assigned_worker: (T::AccountId, WorkerId),
			task_type: TaskType,
			task_owner: T::AccountId,
			task_id: TaskId,
			task: BoundedVec<u8, ConstU32<500>>,
			zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
		},
		/// A completed task has been submitted for verification.
		SubmittedCompletedTask {
			task_id: TaskId,
			assigned_verifier: (T::AccountId, WorkerId),
		},
		/// A resolver has been assigned to determine the correct result after verification failure.
		VerifierResolverAssigned {
			task_id: TaskId,
			assigned_resolver: (T::AccountId, WorkerId),
		},
		/// A completed task has been successfully verified.
		VerifiedCompletedTask { task_id: TaskId },
		/// A completed task has been successfully resolved by the resolver.
		ResolvedCompletedTask { task_id: TaskId },
		/// A task has been reassigned to a new worker.
		TaskReassigned {
			task_id: TaskId,
			assigned_executor: (T::AccountId, WorkerId),
		},
	}

	/// Errors inform users that something went wrong.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::error]
	pub enum Error<T> {
		/// The provided task ID does not exist.
		UnassignedTaskId,
		/// The caller is not the task owner.
		InvalidTaskOwner,
		/// A task must be assigned before it can proceed to the next step.
		RequireAssignedTask,
		/// A verifier must be assigned to the task.
		RequireAssignedVerifier,
		/// A completed task's hash must be provided by the assigned verifier.
		RequireAssignedVerifierCompletedHash,
		/// A resolver must be assigned to review the task.
		RequireAssignedResolver,
		/// No workers are available for the task.
		NoWorkersAvailable,
		/// The task verification process cannot be found.
		TaskVerificationNotFound,
		/// No new workers are available for the task reassignment.
		NoNewWorkersAvailable,
		/// The worker, to which the task should be assigned does not exist.
		WorkerDoesNotExist,
		/// A compute hour deposit is required to schedule or proceed with the task.
		RequireComputeHoursDeposit,
		/// The user has insufficient compute hours balance for the requested deposit.
		InsufficientComputeHours,
		/// The user submitted a ZK task, but has not provided the required files for proof generation
		ZkFilesMissing,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new task and assigns it to a randomly selected worker.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::task_scheduler(task_data.len() as u32))]
		pub fn task_scheduler(
			origin: OriginFor<T>,
			task_type: TaskType,
			task_data: BoundedVec<u8, ConstU32<500>>,
			zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			compute_hours_deposit: Option<u32>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// Ensure that compute_hour_deposit is provided and greater than zero
			let deposit = compute_hours_deposit.ok_or(Error::<T>::RequireComputeHoursDeposit)?;
			ensure!(deposit > 0, Error::<T>::RequireComputeHoursDeposit);

			// Ensure that, if the task type is ZK, the ZK files actually are present
			match task_type {
				cyborg_primitives::task::TaskType::ZK => {
					ensure!(zk_files_cid.is_some(), Error::<T>::ZkFilesMissing)
				}
				_ => {}
			}

			let existing_workers = AccountWorkers::<T>::iter().next().is_some();
			ensure!(existing_workers, Error::<T>::NoWorkersAvailable);

			// Call consume_compute_hours in the payment pallet to deduct the compute hours
			pallet_payment::Pallet::<T>::consume_compute_hours(origin.clone(), deposit)?;

			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.wrapping_add(1));

			let selected_worker = (worker_owner, worker_id);

			if task_type == cyborg_primitives::task::TaskType::Docker {
				ensure!(
					WorkerClusters::<T>::contains_key(&selected_worker),
					Error::<T>::WorkerDoesNotExist
				);
			}

			if task_type == cyborg_primitives::task::TaskType::Executable
				|| task_type == cyborg_primitives::task::TaskType::ZK
			{
				ensure!(
					ExecutableWorkers::<T>::contains_key(&selected_worker),
					Error::<T>::WorkerDoesNotExist
				);
			}

			let task_info = TaskInfo {
				task_type: task_type.clone(),
				task_owner: who.clone(),
				create_block: <frame_system::Pallet<T>>::block_number(),
				metadata: task_data.clone(),
				zk_files_cid: zk_files_cid.clone(),
				time_elapsed: None,
				average_cpu_percentage_use: None,
				result: None,
				compute_hours_deposit,
				consume_compute_hours: None,
			};

			// Assign task to worker and set task owner.
			TaskAllocations::<T>::insert(task_id, selected_worker.clone());
			TaskOwners::<T>::insert(task_id, who.clone());
			Tasks::<T>::insert(task_id, task_info);
			TaskStatus::<T>::insert(task_id, TaskStatusType::Assigned);

			// Emit an event.
			Self::deposit_event(Event::TaskScheduled {
				assigned_worker: selected_worker,
				task_type,
				task_owner: who,
				task_id,
				task: task_data,
				zk_files_cid: zk_files_cid,
			});
			Ok(())
		}

		/// Allows a worker to submit a completed task for verification by a verifier.
		#[pallet::call_index(1)]
		#[pallet::weight(/*<T as pallet::Config>::WeightInfo::submit_completed_task(u32::MAX)*/500000000)]
		pub fn submit_completed_task(
			origin: OriginFor<T>,
			task_id: TaskId,
			completed_hash: H256,
			result: BoundedVec<u8, ConstU32<500>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Retrieve the task details
			let task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;

			// Retrieve compute_hours_deposit and consume_compute_hours from the task
			let compute_hours_deposit = task_info.compute_hours_deposit.unwrap_or(0);
			let consume_compute_hours = task_info.consume_compute_hours.unwrap_or(0);
			// Retrieve task owner
			let task_owner = task_info.task_owner.clone();
			// Retrieve the task status
			let task_status = TaskStatus::<T>::get(task_id);

			// Calculate the refund
			let refund = compute_hours_deposit.saturating_sub(consume_compute_hours);

			// If there is a refund and the task status is `Assigned`, add the refund to the user's compute hours (task_owner)
			if refund > 0 && task_status == Some(TaskStatusType::Assigned) {
				// Update the storage with the new balance
				ComputeHours::<T>::mutate(&task_owner, |balance| {
					*balance = balance.saturating_add(refund);
				});
			}

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

			let mut forbidden_owners: ForbiddenOwners<T::AccountId> = Vec::new();
			forbidden_owners.push(Some(who.clone()));

			let assigned_verifier: (T::AccountId, WorkerId) =
				Self::return_random_worker_of_same_type(task_info.task_type, forbidden_owners, &ver)?;

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

		/// Verifies whether the submitted completed task is correct.
		/// If verification fails, a new resolver is assigned to review the task.
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
			let task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;

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

						let mut forbidden_owners: ForbiddenOwners<T::AccountId> = Vec::new();
						forbidden_owners.push(Some(verification.executor.account.clone()));
						forbidden_owners.push(
							verification
								.verifier
								.clone()
								.map_or(None, |v| Some(v.account)),
						);

						let assigned_resolver = Self::return_random_worker_of_same_type(
							task_info.task_type,
							forbidden_owners,
							verification,
						)?;

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

		/// Resolver finalizes the verification of a task in case of disputes.
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
			let task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;

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

						let mut forbidden_owners: ForbiddenOwners<T::AccountId> = Vec::new();
						forbidden_owners.push(Some(resolver_account));
						forbidden_owners.push(Some(executor.account.clone()));
						forbidden_owners.push(
							verification
								.verifier
								.clone()
								.map_or(None, |v| Some(v.account)),
						);

						let assigned_new_executor = Self::return_random_worker_of_same_type(
							task_info.task_type,
							forbidden_owners,
							verification,
						)?;

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

	impl<T: Config> Pallet<T> {
		pub fn return_random_worker_of_same_type(
			task_type: cyborg_primitives::task::TaskType,
			forbidden_owners: Vec<Option<T::AccountId>>,
			randomness_salt: &Verifications<T::AccountId>,
		) -> Result<(T::AccountId, WorkerId), Error<T>> {
			let workers: Vec<_>;

			match task_type {
				cyborg_primitives::task::TaskType::Docker => {
					workers = WorkerClusters::<T>::iter()
						.filter(|&(_, ref worker)| {
							worker.status == WorkerStatusType::Inactive
								&& !forbidden_owners.contains(&Some(worker.owner.clone()))
						}) // TODO: change Inactive to Active with oracle
						.collect::<Vec<_>>();
				}
				cyborg_primitives::task::TaskType::Executable | cyborg_primitives::task::TaskType::ZK => {
					workers = ExecutableWorkers::<T>::iter()
						.filter(|&(_, ref worker)| {
							worker.status == WorkerStatusType::Inactive
								&& !forbidden_owners.contains(&Some(worker.owner.clone()))
						}) // TODO: change Inactive to Active with oracle
						.collect::<Vec<_>>();
				}
			}

			ensure!(workers.len() > 0, Error::<T>::NoNewWorkersAvailable);

			let random_index =
				(sp_io::hashing::blake2_256(&randomness_salt.encode())[0] as usize) % workers.len();
			let assigned_verifier: (T::AccountId, WorkerId) = workers[random_index].0.clone();
			Ok(assigned_verifier)
		}
	}
}
