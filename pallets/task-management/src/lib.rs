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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::{OriginFor, *};
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

	/// Storage for compute aggregation information (start and end block).
    #[pallet::storage]
    #[pallet::getter(fn compute_aggregations)]
    pub type ComputeAggregations<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        TaskId, 
        (BlockNumberFor<T>, Option<BlockNumberFor<T>>), 
        OptionQuery
    >;

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
		/// A new task has been scheduled and assigned to a worker.
		TaskScheduled {
			assigned_worker: (T::AccountId, WorkerId),
			task_type: TaskType,
			task_kind: TaskKind, 
			task_owner: T::AccountId,
			task_id: TaskId,
			task: BoundedVec<u8, ConstU32<500>>,
			zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
		},
	
		/// A worker confirmed reception of task data and started execution.
		TaskReceptionConfirmed {
			task_id: TaskId,
			who: T::AccountId,
		},
	
		/// A completed task has been submitted by a worker and assigned to a verifier.
		SubmittedCompletedTask {
			task_id: TaskId,
			assigned_verifier: (T::AccountId, WorkerId),
		},
	
		/// A task was successfully verified and completed.
		VerifiedCompletedTask {
			task_id: TaskId,
		},
	
		/// Verifier disagreed with executor, a resolver has been assigned.
		VerifierResolverAssigned {
			task_id: TaskId,
			assigned_resolver: (T::AccountId, WorkerId),
		},
	
		/// A task dispute was resolved successfully (task verified).
		ResolvedCompletedTask {
			task_id: TaskId,
		},
	
		/// Task was reassigned to a new executor after failed dispute resolution.
		TaskReassigned {
			task_id: TaskId,
			assigned_executor: (T::AccountId, WorkerId),
		},
	
		/// Controller/admin requested to stop a running task.
		TaskStopRequested {
			task_id: TaskId,
		},
	
		/// Miner confirmed that they have vacated/reset after stopping.
		MinerVacated {
			task_id: TaskId,
		},
	}

	/// Errors inform users that something went wrong.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::error]
	pub enum Error<T> {

		
		/// No new workers are available for the task reassignment.
		NoNewWorkersAvailable,
		/// The user has insufficient compute hours balance for the requested deposit.
		InsufficientComputeHours,
		InvalidTaskState,
		NotTaskOwner,
		InvalidZKExecution,
		UnexpectedZkFiles,
		InvalidInferenceExecution,
		TaskNotFound,
		// Scheduling errors
		RequireComputeHoursDeposit, // A compute hour deposit is required to schedule or proceed with the task.
		ZkFilesMissing,			// The user submitted a ZK task, but has not provided the required files for proof generation
		NoWorkersAvailable,  // No workers are available for the task.
		WorkerDoesNotExist,  // The worker, to which the task should be assigned does not exist.
	
		// General task errors
		UnassignedTaskId, // The provided task ID does not exist.
		InvalidTaskOwner, // The caller is not the task owner.
		TaskVerificationNotFound, 	// The task verification process cannot be found.

		// Status transition errors
		RequireAssignedTask, // A task must be assigned before it can proceed to the next step.
		RequireRunningTask,
		RequirePendingValidationTask,
		RequireStoppedTask,
		RequireVacatedTask,

		// Verification-specific errors
		RequireAssignedVerifier, // A verifier must be assigned to the task.
		RequireAssignedVerifierCompletedHash, // A completed task's hash must be provided by the assigned verifier.
		RequireAssignedResolver,		// A resolver must be assigned to review the task.
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new task and assigns it to a randomly selected worker.
		/// None -> Assigned
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::task_scheduler(task_data.len() as u32))]
		pub fn task_scheduler(
			origin: OriginFor<T>,
			task_type: TaskType,
			task_kind: TaskKind,
			task_data: BoundedVec<u8, ConstU32<500>>,
			zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			compute_hours_deposit: Option<u32>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
		
			// Validate deposit
			let deposit = compute_hours_deposit.ok_or(Error::<T>::RequireComputeHoursDeposit)?;
			ensure!(deposit > 0, Error::<T>::RequireComputeHoursDeposit);
		
			// Check task_type and task_kind compatibility
			match task_kind {
				TaskKind::NeuroZK => {
					// For NeuroZK, zk_files_cid must be present
					ensure!(zk_files_cid.is_some(), Error::<T>::ZkFilesMissing);
		
					// Allow only Executable or Docker environments
					match task_type {
						TaskType::Executable | TaskType::Docker => {}, // valid
						_ => return Err(Error::<T>::InvalidZKExecution.into()),
					}
				},
				TaskKind::OpenInference => {
					// zk_files_cid should not be present for normal inference
					ensure!(zk_files_cid.is_none(), Error::<T>::UnexpectedZkFiles);
		
					// OpenInference can also be run in Executable or Docker
					match task_type {
						TaskType::Executable | TaskType::Docker => {}, // valid
						_ => return Err(Error::<T>::InvalidInferenceExecution.into()),
					}
				},
			}
		
			// Ensure there's at least one worker registered
			let existing_workers = AccountWorkers::<T>::iter().next().is_some();
			ensure!(existing_workers, Error::<T>::NoWorkersAvailable);
		
			// Consume compute hours from payment pallet
			pallet_payment::Pallet::<T>::consume_compute_hours(origin.clone(), deposit)?;
		
			// Generate task ID
			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.wrapping_add(1));
		
			let selected_worker = (worker_owner, worker_id);
		
			// Ensure worker is registered in the appropriate registry
			match task_type {
				TaskType::Docker => {
					ensure!(
						WorkerClusters::<T>::contains_key(&selected_worker),
						Error::<T>::WorkerDoesNotExist
					);
				},
				TaskType::Executable => {
					ensure!(
						ExecutableWorkers::<T>::contains_key(&selected_worker),
						Error::<T>::WorkerDoesNotExist
					);
				},
			}
		
			let task_info = TaskInfo::<T::AccountId, BlockNumberFor<T>> {
				task_owner: who.clone(),
				create_block: <frame_system::Pallet<T>>::block_number(),
				metadata: task_data.clone(),
				zk_files_cid: zk_files_cid.clone(),
				time_elapsed: None,
				average_cpu_percentage_use: None,
				task_type: task_type.clone(),
				task_kind: task_kind.clone(),
				result: None,
				compute_hours_deposit: Some(deposit),
				consume_compute_hours: None,
				task_status: TaskStatusType::Assigned,
			};
		
			TaskAllocations::<T>::insert(task_id, selected_worker.clone());
			TaskOwners::<T>::insert(task_id, who.clone());
			Tasks::<T>::insert(task_id, task_info);
			TaskStatus::<T>::insert(task_id, TaskStatusType::Assigned);
		
			Self::deposit_event(Event::TaskScheduled {
				assigned_worker: selected_worker,
				task_type,
				task_kind, 
				task_owner: who,
				task_id,
				task: task_data,
				zk_files_cid,
			});
		
			Ok(())
		}

		/// Miner confirms that it has gathered the data and is starting task execution.
		///
		/// Allowed only if task is still `Assigned`.
		/// Changes task state to `Running` and starts aggregation of resource usage.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn confirm_task_reception(
			origin: OriginFor<T>,
			task_id: TaskId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
		
			// Load task
			let mut task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
		
			// Check that caller is the assigned worker
			let assigned_worker = TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
			ensure!(assigned_worker.0 == who, Error::<T>::InvalidTaskOwner);
		
			// Task must currently be `Assigned`
			ensure!(
				task_info.task_status == TaskStatusType::Assigned,
				Error::<T>::RequireAssignedTask
			);
		
			// Transition task to `Running`
			task_info.task_status = TaskStatusType::Running;
			TaskStatus::<T>::insert(task_id,TaskStatusType::Running);

		
			// Store back the updated task info
			Tasks::<T>::insert(task_id, task_info);
		
			// Start compute aggregation: record starting block
			ComputeAggregations::<T>::insert(
				task_id,
				(<frame_system::Pallet<T>>::block_number(), None::<BlockNumberFor<T>>),
			);
		
			// Emit event (you can define a new event like TaskReceptionConfirmed if needed)
			Self::deposit_event(Event::TaskReceptionConfirmed {
				task_id,
				who,
			});
		
			Ok(())
		}

		/// Worker submits the completed task result.
		/// Moves task to `PendingValidation`, assigns a verifier.
		///
		/// Allowed only if task is `Running`.
		#[pallet::call_index(2)]
		#[pallet::weight(/*<T as pallet::Config>::WeightInfo::submit_completed_task(u32::MAX)*/500000000)]
		pub fn submit_completed_task(
			origin: OriginFor<T>,
			task_id: TaskId,
			completed_hash: H256,
			result: BoundedVec<u8, ConstU32<500>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Load the task info
			let mut task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
		
			// Validate caller is the assigned worker
			let task_assignee = TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
			ensure!(task_assignee.0 == who, Error::<T>::InvalidTaskOwner);
		
			// Task must be `Running` (confirmed reception)
			ensure!(
				task_info.task_status == TaskStatusType::Running,
				Error::<T>::RequireAssignedTask
			);
		
			// Refund compute hours if over-allocated
			let compute_hours_deposit = task_info.compute_hours_deposit.unwrap_or(0);
			let consume_compute_hours = task_info.consume_compute_hours.unwrap_or(0);
			let refund = compute_hours_deposit.saturating_sub(consume_compute_hours);
			if refund > 0 && task_info.task_status == TaskStatusType::Assigned {
				// Update the storage with the new balance
				ComputeHours::<T>::mutate(&task_info.task_owner, |balance| {
					*balance = balance.saturating_add(refund);
				});
			}
		
			// Update task info: result saved
			task_info.task_status = TaskStatusType::PendingValidation;
			task_info.result = Some(result.clone());
			Tasks::<T>::insert(task_id, task_info.clone());
			TaskStatus::<T>::insert(task_id, TaskStatusType::PendingValidation);
		
			// Prepare verifications
			let mut verifications = Verifications {
				executor: VerificationHashes {
					account: who.clone(),
					completed_hash: Some(completed_hash),
				},
				verifier: None,
				resolver: None,
			};
		
			// Build forbidden owners: executor should not be verifier
			
			let mut forbidden_owners: ForbiddenOwners<T::AccountId> = Vec::new();
			forbidden_owners.push(Some(who.clone()));
		
			// Randomly assign verifier
			let assigned_verifier = Self::return_random_worker_of_same_type(
				task_info.task_type.clone(),
				forbidden_owners,
				&verifications,
			)?;
		
			verifications.verifier = Some(VerificationHashes {
				account: assigned_verifier.0.clone(),
				completed_hash: None,
			});

			Tasks::<T>::mutate(task_id, |task| {
				if let Some(ref mut raw_task) = task {
					raw_task.result = Some(result);
				}
			});

		
			// Save verifications
			TaskVerifications::<T>::insert(task_id, verifications);
		
			// Stop compute aggregation: mark block number
			ComputeAggregations::<T>::mutate(task_id, |record| {
				if let Some((start_block, _)) = record {
					*record = Some((*start_block, Some(<frame_system::Pallet<T>>::block_number())));
				}
			});
		
			// Emit event
			Self::deposit_event(Event::SubmittedCompletedTask {
				task_id,
				assigned_verifier,
			});
		
			Ok(())
		}

		/// Verifies whether the submitted completed task is correct.
		/// If verification fails, a new resolver is assigned to review the task.
		/// Task result verified correctly Change to Completed or not Still PendingValidation
		/// Task verification failed, reassignment ongoing (Still PendingValidation)
		/// Pending -> Completed
		#[pallet::call_index(3)]
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

		/// Resolver resolves a task dispute after a verifier-executor disagreement.
		///
		/// If resolver's provided hash matches executor or verifier, task is completed.
		/// Otherwise, task is reassigned to a new executor.
		///
		/// Only allowed if task is in `PendingValidation` and resolver assigned.
		#[pallet::call_index(4)]
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

		/// signals the miner to exit task execution and reset itself
		/// Admin will make status to stopped
		/// Completed -> Stopped
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn stop_task_and_vacate_miner(origin: OriginFor<T>,task_id: TaskId,)->DispatchResult{
			ensure_signed(origin)?; // anyone controlling can request stop

            let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

            // Ensure task is running.
            ensure!(task.task_status == TaskStatusType::Running, Error::<T>::InvalidTaskState);

            // Change task state to Stopped.
            task.task_status = TaskStatusType::Stopped;
            Tasks::<T>::insert(task_id, task);

            // Mark end of compute aggregation.
            ComputeAggregations::<T>::mutate(task_id, |record| {
                if let Some((start, _)) = record {
                    *record = Some((*start, Some(<frame_system::Pallet<T>>::block_number())));
                }
            });

            // Emit event.
            Self::deposit_event(Event::TaskStopRequested { task_id });

            Ok(())
			
		}
		
		/// miner confirms that it has reset itself
		/// Stopped to vacated 
		/// With some Logic Expired
		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn confirm_miner_vacation(origin:OriginFor<T>,task_id:TaskId)->DispatchResult{
			let who = ensure_signed(origin)?;

            let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

            // Ensure task owner is confirming.
            ensure!(task.task_owner == who, Error::<T>::NotTaskOwner);

            // Ensure task is stopped.
            ensure!(task.task_status == TaskStatusType::Stopped, Error::<T>::InvalidTaskState);

            // Move to Vacated state.
            task.task_status = TaskStatusType::Vacated;
            Tasks::<T>::insert(task_id, task);

            // Emit event.
            Self::deposit_event(Event::MinerVacated { task_id });

            Ok(())
		}

	
	}

	impl<T: Config> Pallet<T> {
		/// Returns a random available worker of the appropriate execution type (Docker or Executable),
		/// avoiding the given forbidden owners.
		pub fn return_random_worker_of_same_type(
			task_type: cyborg_primitives::task::TaskType,
			forbidden_owners: Vec<Option<T::AccountId>>,
			randomness_salt: &Verifications<T::AccountId>,
		) -> Result<(T::AccountId, WorkerId), Error<T>> {
			let workers: Vec<_> = match task_type {
				cyborg_primitives::task::TaskType::Docker => {
					WorkerClusters::<T>::iter()
						.filter(|&(_, ref worker)| {
							worker.status == WorkerStatusType::Inactive &&
								!forbidden_owners.contains(&Some(worker.owner.clone()))
						})
						.collect()
				},
				cyborg_primitives::task::TaskType::Executable => {
					ExecutableWorkers::<T>::iter()
						.filter(|&(_, ref worker)| {
							worker.status == WorkerStatusType::Inactive &&
								!forbidden_owners.contains(&Some(worker.owner.clone()))
						})
						.collect()
				},
			};

			ensure!(!workers.is_empty(), Error::<T>::NoNewWorkersAvailable);

			let random_index = (sp_io::hashing::blake2_256(&randomness_salt.encode())[0] as usize) % workers.len();
			let assigned_verifier = workers[random_index].0.clone(); // (AccountId, WorkerId)
			Ok(assigned_verifier)
		}
	}
}
