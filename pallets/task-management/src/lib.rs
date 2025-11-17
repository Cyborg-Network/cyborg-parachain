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

use cyborg_primitives::miner::{MinerId, MinerType};
pub use cyborg_primitives::task::*;
use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use pallet_edge_connect::SuspensionReason;

use scale_info::prelude::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::PostDispatchInfo;
	use frame_support::sp_runtime::Saturating;
	use frame_support::traits::Currency;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::{OriginFor, *};
	use pallet_edge_connect::PenaltyReason;
	use pallet_timestamp as timestamp;
	// use pallet_edge_connect::AccountMiners;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_edge_connect::Config + pallet_payment::Config + timestamp::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/reference_docs/frame_runtime_types/index.html>
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;

		/// Number of blocks to wait for task confirmation before penalizing miner
		type TaskConfirmationTimeout: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Status of tasks within the system.
	#[pallet::storage]
	pub type TaskStatus<T: Config> = StorageMap<_, Twox64Concat, TaskId, TaskStatusType, OptionQuery>;

	/// Allocation of tasks to miners.
	#[pallet::storage]
	pub type TaskAllocations<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, MinerId, OptionQuery>;

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

	#[pallet::storage]
	pub type GatekeeperAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type TaskRateLimits<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		(BlockNumberFor<T>, u32), // (last_block, count)
		ValueQuery,
	>;

	/// Storage for compute aggregation information (start and end block).
	#[pallet::storage]
	#[pallet::getter(fn compute_aggregations)]
	pub type ComputeAggregations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		TaskId,
		(BlockNumberFor<T>, Option<BlockNumberFor<T>>),
		OptionQuery,
	>;

	#[pallet::storage]
	pub type ModelHashes<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], T::Hash, OptionQuery>;

	#[pallet::storage]
	pub type TaskAssignmentBlock<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, BlockNumberFor<T>, OptionQuery>;

	#[pallet::storage]
	pub type PendingTaskConfirmations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,                 // Timeout block
		BoundedVec<TaskId, ConstU32<100>>, // Tasks expiring at this block
		ValueQuery,
	>;

	/// Pallets use events to inform users when important changes are made.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new task has been scheduled and assigned to a miner.
		TaskScheduled {
			assigned_miner: (T::AccountId, MinerId),
			task_kind: TaskKind<BlockNumberFor<T>>,
			task_owner: T::AccountId,
			task_id: TaskId,
		},

		/// A miner confirmed reception of task data and started execution.
		TaskReceptionConfirmed {
			task_id: TaskId,
			who: T::AccountId,
		},

		/// Controller/admin requested to stop a running task.
		TaskStopRequested {
			task_id: TaskId,
		},

		/// Miner confirmed that they have vacated/reset after stopping.
		MinerVacated {
			task_id: TaskId,
		},
		ModelHashRegistered(Vec<u8>, T::Hash),
		ModelHashQueried(Vec<u8>, T::Hash),

		/// Event emitted when a task is manually reset by admin
		TaskManuallyReset {
			task_id: TaskId,
			reset_by: Option<T::AccountId>,
			previous_status: TaskStatusType,
			reason: ResetReason,
		},
	}

	/// Errors inform users that something went wrong.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::error]
	pub enum Error<T> {
		InvalidTaskState,
		NotTaskOwner,
		UnexpectedZkFiles,
		InvalidModelIdLength,
		NotGatekeeper,
		InvalidModelId,
		NotAssignedMiner,
		// Scheduling errors
		RequireComputeHoursDeposit, // A compute hour deposit is required to schedule or proceed with the task.
		ZkFilesMissing, // The user submitted a ZK task, but has not provided the required files for proof generation

		// General task errors
		TaskNotFound,         // The provided task ID does not exist.
		InvalidTaskOwner,         // The caller is not the task owner.
		TaskVerificationNotFound, // The task verification process cannot be found.

		// Status transition errors
		RequireAssignedTask, // A task must be assigned before it can proceed to the next step.

		// Verification-specific errors
		RequireAssignedVerifier, // A verifier must be assigned to the task.

		/// Account has exceeded task submission rate limit
		RateLimitExceeded,
		ModelAlreadyRegistered,
		ModelNotFound,
		TaskReceptionAlreadyConfirmed, // Task reception was already confirmed
		/// Error indicating that the miner does not exist
		MinerDoesNotExist,
		/// Error indicating that the miner is busy
		MinerIsBusy,
		/// Error indicating insufficient reputation
		InsufficientReputation,
		/// Error indicating that the miner is inactive
		MinerIsInactive,
		/// Error indicating that the miner is suspended
		MinerSuspended,

		/// Task cannot be reset in its current state
		TaskNotResettable,
		/// Miner reset failed
		MinerResetFailed,
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
	pub enum ResetReason {
		MinerUnresponsive,
		TaskTimeout,
		SystemError,
		ManualIntervention,
		Other,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			if let Err(e) = Self::check_task_confirmation_timeouts() {
				log::error!("Error checking task confirmation timeouts: {:?}", e);
			}
			T::DbWeight::get().reads_writes(1, 1)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
where
    <<T as pallet_payment::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance:
        TryFrom<u64>, {
		/// Creates a new task and assigns it to a randomly selected miner.
		/// None -> Assigned
		// TODO calculate actual weight from the length of the inputs 
		#[pallet::call_index(0)]
		#[pallet::weight({<T as pallet::Config>::WeightInfo::task_scheduler_nzk(500)})]
		pub fn task_scheduler(
			origin: OriginFor<T>,
			// TODO If the gatekeeper submits the task we need to keep track of which user submitted the task and process the request differently
			// TODO requesting_user: Option<Some data that identifies the user>,
			task_kind: TaskSubmissionData,
			miner_owner: T::AccountId,
			miner_id: MinerId,
			compute_hours_deposit: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Determine miner type based on task kind
			let miner_type = match task_kind {
				TaskSubmissionData::NeuroZK(_) |
				TaskSubmissionData::OpenInference(_) |
				TaskSubmissionData::FlashInfer(_) => {
					MinerType::Edge
				}
				TaskSubmissionData::CyCloud(_) => {
					MinerType::Cloud
				},
			};

			// Check if the miner can accept tasks using the new status system
			let miner_key =  miner_id.clone();
			let miner = pallet_edge_connect::Pallet::<T>::get_miner(&miner_key, &miner_type)
                  .ok_or(pallet_edge_connect::Error::<T>::MinerDoesNotExist)?;

			if !miner.is_eligible_for_tasks() {
				return Err(Error::<T>::MinerIsBusy.into());
			}

			// Check if the miner exists, and if its status allows for task execution
			pallet_edge_connect::Pallet::<T>::check_miner_status(
				&miner_id.clone(),
				&miner_type,
			).map_err(|_| pallet_edge_connect::Error::<T>::MinerDoesNotExist)?;

			let pays_fee = if let Some(gatekeeper) = GatekeeperAccount::<T>::get() {
				if who == gatekeeper {
					Pays::No
				} else {
					Pays::Yes
				}
			} else {
				Pays::Yes
			};

			// Validate deposit
			let deposit = compute_hours_deposit.ok_or(Error::<T>::RequireComputeHoursDeposit)?;
			ensure!(deposit > 0, Error::<T>::RequireComputeHoursDeposit);

			let deposit = compute_hours_deposit.ok_or(Error::<T>::RequireComputeHoursDeposit)?;

            // Consume compute hours from payment pallet
			pallet_payment::Pallet::<T>::consume_compute_hours(origin.clone(), deposit)?;

			// Generate task ID
			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.wrapping_add(1));

			let selected_miner = (miner_owner, miner_id.clone());
			let task_kind = TaskKind::from_submission(task_kind);

			let task_info = TaskInfo::<T::AccountId, BlockNumberFor<T>> {
				task_owner: who.clone(),
				create_block: <frame_system::Pallet<T>>::block_number(),
				time_elapsed: None,
				average_cpu_percentage_use: None,
				task_kind: task_kind.clone(),
				result: None,
				compute_hours_deposit: Some(deposit),
				consume_compute_hours: None,
				task_status: TaskStatusType::Assigned,
			};

			let timeout_block = <frame_system::Pallet<T>>::block_number()
				.saturating_add(T::TaskConfirmationTimeout::get());
			PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
				tasks.try_push(task_id).expect("Task queue bounded to 100 per block");
			});

			TaskAllocations::<T>::insert(task_id, miner_id.clone());
			TaskOwners::<T>::insert(task_id, who.clone());
			Tasks::<T>::insert(task_id, task_info);
			TaskStatus::<T>::insert(task_id, TaskStatusType::Assigned);
			TaskAssignmentBlock::<T>::insert(task_id, <frame_system::Pallet<T>>::block_number());

			pallet_edge_connect::Pallet::<T>::update_miner_status(
				&miner_id,
				miner_type.clone(),
				false,
			)?;

			pallet_edge_connect::Pallet::<T>::update_miner_current_task(
				&miner_id,
				&miner_type.clone(),
				Some(task_id),
			)?;

			Self::deposit_event(Event::TaskScheduled {
				assigned_miner: selected_miner,
				task_kind,
				task_owner: who,
				task_id,
			});

			Ok(PostDispatchInfo {
				actual_weight: None,
				pays_fee,
			})
		}

		/// Miner confirms that it has gathered the data and is starting task execution.
		///
		/// Allowed only if task is still `Assigned`.
		/// Changes task state to `Running` and starts aggregation of resource usage.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_task_reception())]
        pub fn confirm_task_reception(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Load task
            let mut task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

            // Check that caller is the assigned worker
            // let miner_id = TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;
            // ensure!(miner_id == task_id, Error::<T>::InvalidTaskOwner);



            // If task is already running, return specific error
            if task_info.task_status == TaskStatusType::Running {
                return Err(Error::<T>::TaskReceptionAlreadyConfirmed.into());
            }

            // Task must currently be `Assigned`
            ensure!(
                task_info.task_status == TaskStatusType::Assigned,
                Error::<T>::RequireAssignedTask
            );

            task_info.task_status = TaskStatusType::Running;
            TaskStatus::<T>::insert(task_id, TaskStatusType::Running);
            Tasks::<T>::insert(task_id, task_info);

            ComputeAggregations::<T>::insert(
                task_id,
                (
                    <frame_system::Pallet<T>>::block_number(),
                    None::<BlockNumberFor<T>>,
                ),
            );

            Self::deposit_event(Event::TaskReceptionConfirmed { task_id, who });

			   // If confirmation succeeds, remove from pending confirmations
			   if let Some(assigned_block) = TaskAssignmentBlock::<T>::get(task_id) {
				let timeout_block = assigned_block.saturating_add(T::TaskConfirmationTimeout::get());
				PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
					if let Some(pos) = tasks.iter().position(|&id| id == task_id) {
						tasks.swap_remove(pos);
					}
				});
			}

            Ok(())
        }

		//
		/// signals the miner to exit task execution and reset itself
		/// Admin will make status to stopped
		/// RUnning -> Stopped
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::stop_task_and_vacate_miner())]
		pub fn stop_task_and_vacate_miner(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			ensure_signed(origin)?; // anyone controlling can request stop

			let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

			// Ensure task is running.
			ensure!(
				task.task_status == TaskStatusType::Running,
				Error::<T>::InvalidTaskState
			);

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
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_miner_vacation())]
		pub fn confirm_miner_vacation(origin: OriginFor<T>, task_id: TaskId, miner_type: MinerType) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;
			let miner_id = TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

			// Ensure the caller is the miner who was assigned the task
			// ensure!(assigned_miner.0 == who, Error::<T>::NotAssignedMiner);

			// Ensure task is stopped.
			ensure!(
				task.task_status == TaskStatusType::Stopped,
				Error::<T>::InvalidTaskState
			);

			// Move to Vacated state.
			task.task_status = TaskStatusType::Vacated;
			Tasks::<T>::insert(task_id, task);

			// Update the miner status back to active
			pallet_edge_connect::Pallet::<T>::update_miner_status(
				&miner_id,
				// This needs to be changed after the miners have unique IDs
				miner_type,
				true,  // set to available
			)?;

			// Emit event.
			Self::deposit_event(Event::MinerVacated { task_id });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_gatekeeper())]
		pub fn set_gatekeeper(
			origin: OriginFor<T>,
			new_gatekeeper: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			GatekeeperAccount::<T>::put(new_gatekeeper);
			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_model_hash())]
		pub fn register_model_hash(
			origin: OriginFor<T>,
			model_id: Vec<u8>,
			model_hash: T::Hash,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			let gatekeeper = GatekeeperAccount::<T>::get().ok_or(Error::<T>::NotGatekeeper)?;
			ensure!(_sender == gatekeeper, Error::<T>::NotGatekeeper);

			// Validate model_id length
			ensure!(model_id.len() == 32usize, Error::<T>::InvalidModelIdLength);

			// Convert to [u8; 32]
			let model_id_fixed: [u8; 32] = model_id
				.try_into()
				.map_err(|_| Error::<T>::InvalidModelIdLength)?;

			// Ensure it’s not already registered
			ensure!(
				!ModelHashes::<T>::contains_key(&model_id_fixed),
				Error::<T>::ModelAlreadyRegistered
			);

			// Store it
			ModelHashes::<T>::insert(&model_id_fixed, model_hash);

			// Emit event
			Self::deposit_event(Event::ModelHashRegistered(
				model_id_fixed.to_vec(),
				model_hash,
			));
			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::get_model_hash())]
		pub fn get_model_hash(origin: OriginFor<T>, model_id: Vec<u8>) -> DispatchResult {
			let _ = ensure_signed(origin)?; // Anyone can call

			ensure!(model_id.len() == 32, Error::<T>::InvalidModelIdLength);

			let model_id_fixed: [u8; 32] = model_id
				.try_into()
				.map_err(|_| Error::<T>::InvalidModelIdLength)?;

			let model_hash = ModelHashes::<T>::get(&model_id_fixed).ok_or(Error::<T>::ModelNotFound)?;

			Self::deposit_event(Event::ModelHashQueried(model_id_fixed.to_vec(), model_hash));
			Ok(())
		}

		/// Reset a stuck task and its associated miner (sudo only)
        /// This allows manual intervention for tasks that are stuck in Assigned, Running, or Stopped states
        #[pallet::call_index(9)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::reset_task())]
        pub fn reset_task(
           origin: OriginFor<T>,
           task_id: TaskId,
           miner_type: MinerType,
           reason: ResetReason,
         ) -> DispatchResult {
			// Only root can call this function
			ensure_root(origin)?;

            // Get task information
            let task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;
            let previous_status = task.task_status.clone();

            // Check if task is in a resettable state
            if !matches!(
                task.task_status,
                TaskStatusType::Assigned | TaskStatusType::Running | TaskStatusType::Stopped
             ) {
                 return Err(Error::<T>::TaskNotResettable.into());
             }

            // Get assigned miner
            let miner_id = TaskAllocations::<T>::get(task_id)
               .ok_or(Error::<T>::TaskNotFound)?;

			// Store the assigned block
			let assigned_block = TaskAssignmentBlock::<T>::get(task_id);

            // Reset miner status
            Self::reset_miner_for_task(&miner_id, miner_type.clone(), &task_id)?;

            // Clean up task storage
            Tasks::<T>::remove(task_id);
            TaskAllocations::<T>::remove(task_id);
            TaskStatus::<T>::remove(task_id);
            TaskAssignmentBlock::<T>::remove(task_id);
            ComputeAggregations::<T>::remove(task_id);

            // Remove from pending confirmations if present
            if let Some(assigned_block) = assigned_block{
              let timeout_block = assigned_block.saturating_add(T::TaskConfirmationTimeout::get());

                PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
                   if let Some(pos) = tasks.iter().position(|&id| id == task_id) {
                       tasks.swap_remove(pos);
               }
            });
        }

        Self::deposit_event(Event::TaskManuallyReset {
             task_id,
             reset_by: None,
             previous_status,
             reason,
          });

       Ok(())
        }
	}

	impl<T: Config> Pallet<T> {
		#[allow(dead_code)]
		fn check_rate_limit(who: &T::AccountId) -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();
			let (last_block, count) = TaskRateLimits::<T>::get(who);

			// Reset counter if it's a new block
			let new_count = if current_block == last_block {
				count + 1
			} else {
				1
			};

			// Update storage
			TaskRateLimits::<T>::insert(who, (current_block, new_count));

			// Allow up to 5 tasks per block per account
			if new_count > 5 {
				Err(Error::<T>::RateLimitExceeded.into())
			} else {
				Ok(())
			}
		}

		/// Check for expired task confirmations and penalize miners
		pub fn check_task_confirmation_timeouts() -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Only check tasks that are due at or before current_block
			for (timeout_block, task_ids) in PendingTaskConfirmations::<T>::iter() {
				if timeout_block > current_block {
					break;
				}

				for task_id in task_ids.iter() {
					// Check if task is still in Assigned state
					if let Some(status) = TaskStatus::<T>::get(task_id) {
						if status != TaskStatusType::Assigned {
							continue;
						}

						// Get assigned miner and penalize
						if let Some(miner_id) = TaskAllocations::<T>::get(task_id) {
							pallet_edge_connect::Pallet::<T>::apply_penalty(
								&miner_id.clone(),
								&MinerType::Edge,
								20,
								PenaltyReason::LateResponse,
							)?;

							pallet_edge_connect::Pallet::<T>::suspend_miners(
								&miner_id.clone(),
								&MinerType::Edge,
								1000u32.into(),
								SuspensionReason::TaskConfirmationTimeout,
							)?;

							// Clean up task state
							TaskStatus::<T>::remove(task_id);
							TaskAssignmentBlock::<T>::remove(task_id);
						}
					}
				}

				// Remove processed block from storage
				PendingTaskConfirmations::<T>::remove(timeout_block);
			}

			Ok(())
		}

		/// Reset miner associated with a task
		fn reset_miner_for_task(
			miner_key: &MinerId,
			miner_type: MinerType,
			task_id: &TaskId,
		) -> DispatchResult {
			// Get current miner state
			let miner = pallet_edge_connect::Pallet::<T>::get_miner(miner_key, &miner_type)
				.ok_or(Error::<T>::MinerResetFailed)?;

			// Only reset if the miner is currently working on this task
			if let Some(current_task) = miner.current_task {
				if current_task == *task_id {
					// Reset miner to available status
					pallet_edge_connect::Pallet::<T>::update_miner_status(
						miner_key,
						miner_type.clone(),
						true, // set to available
					)
					.map_err(|_| Error::<T>::MinerResetFailed)?;

					// Clear current task
					pallet_edge_connect::Pallet::<T>::update_miner_current_task(miner_key, &miner_type, None)
						.map_err(|_| Error::<T>::MinerResetFailed)?;

					// If miner was suspended due to this task, lift suspension
					if miner.is_suspended() {
						let _ = pallet_edge_connect::Pallet::<T>::lift_suspension(miner_key, &miner_type);
					}
				}
			}

			Ok(())
		}

		/// Helper function to get all stuck tasks
		pub fn get_stuck_tasks(
			current_block: BlockNumberFor<T>,
			timeout_blocks: BlockNumberFor<T>,
		) -> Vec<(TaskId, TaskInfo<T::AccountId, BlockNumberFor<T>>)> {
			let mut stuck_tasks = Vec::new();

			for (task_id, task_info) in Tasks::<T>::iter() {
				match task_info.task_status {
					TaskStatusType::Assigned => {
						// Check if task assignment has timed out
						if let Some(assigned_block) = TaskAssignmentBlock::<T>::get(task_id) {
							if current_block.saturating_sub(assigned_block) > timeout_blocks {
								stuck_tasks.push((task_id, task_info));
							}
						}
					}
					TaskStatusType::Running => {
						// Check if task has been running for too long without progress
						if let Some((start_block, _)) = ComputeAggregations::<T>::get(task_id) {
							if current_block.saturating_sub(start_block)
								> timeout_blocks.saturating_mul(10u32.into())
							{
								stuck_tasks.push((task_id, task_info));
							}
						}
					}
					TaskStatusType::Stopped => {
						// Stopped tasks that haven't been vacated are considered stuck
						stuck_tasks.push((task_id, task_info));
					}
					_ => {} // Vacated tasks are not considered stuck
				}
			}

			stuck_tasks
		}
	}

	impl<T: Config + timestamp::Config> NzkTaskInfoHandler<T::AccountId, TaskId, BlockNumberFor<T>>
		for Pallet<T>
	{
		// Implementation of the NzkTaskInfoHandler trait, which provides methods for accessing NZK task information.
		fn get_nzk_task(task_key: TaskId) -> Option<TaskInfo<T::AccountId, BlockNumberFor<T>>> {
			Tasks::<T>::get(task_key)
		}

		// Implementation of the NzkTaskInfoHandler trait, which provides methods for NZK task information.
		fn update_nzk_task(task_key: TaskId, task: TaskInfo<T::AccountId, BlockNumberFor<T>>) {
			Tasks::<T>::insert(task_key, task);
		}
	}
}
