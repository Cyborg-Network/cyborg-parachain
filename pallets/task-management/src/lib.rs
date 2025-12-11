#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock1;
//mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

pub use cyborg_primitives::task::*;
use cyborg_primitives::{
	miner::{Miner, MinerId, MinerType, OperationalStatus},
	payment::{PaymentDetails, PaymentMode, PaymentPurpose, PaymentRates},
};
use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_edge_connect::{MinerInfoHandler, SuspensionReason};
use pallet_payment::{AssetIdOf, BalanceOf, PaymentDetailsOf};
use scale_info::prelude::vec::Vec;

pub type TaskInfoOf<T> =
	TaskInfo<<T as frame_system::Config>::AccountId, BlockNumberFor<T>, AssetIdOf<T>, BalanceOf<T>>;
pub type StoppedTaskInfoOf<T> = (TaskId, PaymentDetailsOf<T>);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, sp_runtime::Saturating, traits::Currency,
	};
	use frame_system::pallet_prelude::{ensure_root, ensure_signed, OriginFor};
	use pallet_edge_connect::PenaltyReason;
	use pallet_timestamp as timestamp;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_edge_connect::Config
		+ pallet_payment::Config
		+ timestamp::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;

		/// Number of blocks to wait for task confirmation before penalizing miner
		type TaskConfirmationTimeout: Get<BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Allocation of tasks to miners.
	#[pallet::storage]
	pub type TaskAllocations<T: Config> = StorageMap<_, Twox64Concat, TaskId, MinerId, OptionQuery>;

	/// The next task ID to be assigned.
	#[pallet::storage]
	pub type NextTaskId<T: Config> = StorageValue<_, TaskId, ValueQuery>;

	/// Task metadata and information.
	#[pallet::storage]
	pub type Tasks<T: Config> = StorageMap<_, Identity, TaskId, TaskInfoOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type GatekeeperAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type ModelHashes<T: Config> =
		StorageMap<_, Blake2_128Concat, [u8; 32], T::Hash, OptionQuery>;

	#[pallet::storage]
	pub type TaskAssignmentBlock<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, BlockNumberFor<T>, OptionQuery>;

	#[pallet::storage]
	pub type PendingTaskConfirmations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<TaskId, ConstU32<100>>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TaskScheduled {
			assigned_miner: MinerId,
			task_kind: TaskKind,
			task_owner: T::AccountId,
			task_id: TaskId,
		},
		TaskReceptionConfirmed {
			task_id: TaskId,
			who: T::AccountId,
		},
		/// A miner confirmed the task reception, but failed to run the task
		TaskReceptionFailed {
			task_id: TaskId,
			who: T::AccountId,
		},

		/// Controller/admin requested to stop a running task.
		TaskStopRequested {
			task_id: TaskId,
		},
		MinerVacated {
			task_id: TaskId,
		},
		ModelHashRegistered(Vec<u8>, T::Hash),
		ModelHashQueried(Vec<u8>, T::Hash),
		TasksStoppedForExpiredPayments {
			count: u32,
			stopped_tasks: Vec<StoppedTaskInfoOf<T>>,
		},
		PaymentInitiatedForTask {
			owner: T::AccountId,
			id: TaskId,
			payment: PaymentDetailsOf<T>,
		},
		TaskManuallyReset {
			id: TaskId,
			reset_by: Option<T::AccountId>,
			previous_status: TaskStatusType,
			reason: ResetReason,
		},
		TaskTerminated {
			id: TaskId,
			owner: T::AccountId,
			refund_amount: BalanceOf<T>,
			execution_time: BlockNumberFor<T>,
		},
		TaskCancelled {
			id: TaskId,
			owner: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTaskState,
		NotAssignedMiner,
		// Scheduling errors
		RequireComputeHoursDeposit, /* A compute hour deposit is required to schedule or
		                             * proceed with the task. */
		ZkFilesMissing, /* The user submitted a ZK task, but has not provided the required
		                 * files for proof generation */

		// General task errors
		TaskNotFound,             // The provided task ID does not exist.
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
		NotAuthorized,
		TaskNotAllocated,
		NotGatekeeper,
		InvalidModelIdLength,
		TaskNotResettable,
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

			let mut weight = T::DbWeight::get().reads_writes(1, 1);

			// Stop tasks for users with expired payments
			if let Err(e) = Self::stop_tasks_for_expired_payments() {
				log::error!("Error stopping tasks for expired payments: {:?}", e);
			}

			weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({<T as pallet::Config>::WeightInfo::task_scheduler_nzk(500)})]
		pub fn schedule(
			origin: OriginFor<T>,
			submission: TaskKind,
			miner_id: MinerId, // TODO: Randomize miner selection
			mode: PaymentMode,
			asset: AssetIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let miner_type = match submission {
				TaskKind::OpenInference(_) | TaskKind::FlashInfer(_) => MinerType::Edge,
				TaskKind::CyCloud(_) => MinerType::Cloud,
			};

			pallet_edge_connect::Pallet::<T>::check_miner_available(&miner_id, &miner_type)?;

			let now = frame_system::Pallet::<T>::block_number();

			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.saturating_add(1));

			let details = PaymentDetailsOf::<T> {
				begin: Zero::zero(),
				expiry: Zero::zero(),
				asset,
				amount: Zero::zero(),
				mode,
			};
			// Reserve task execution cost
			let details = pallet_payment::Pallet::<T>::reserve(
				&who,
				PaymentPurpose::TaskExecution(task_id),
				details,
			)?;

			let timeout_block = now.saturating_add(T::TaskConfirmationTimeout::get());

			// This should never fail but if it does, we handle it gracefully
			PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
				tasks.try_push(task_id).map_err(|_| Error::<T>::RateLimitExceeded)
			})?;

			//let task_kind = TaskKind::from_submission(submission);

			let task_info = TaskInfoOf::<T> {
				task_owner: who.clone(),
				create_block: now,
				time_elapsed: None,
				average_cpu_percentage_use: None,
				task_kind: submission.clone(),
				result: None,
				task_status: TaskStatusType::Assigned,
				payment: details.clone(),
			};

			let existing_miner =
				pallet_edge_connect::Pallet::<T>::get_miner(&miner_id, &miner_type)
					.ok_or(Error::<T>::MinerDoesNotExist)?;

			let miner = Miner {
				operational_status: OperationalStatus::TaskAssigned,
				status_last_updated: now,
				current_task: Some(task_id),
				..existing_miner
			};

			TaskAllocations::<T>::insert(task_id, miner_id.clone());
			Tasks::<T>::insert(task_id, task_info);
			TaskAssignmentBlock::<T>::insert(task_id, now);

			pallet_edge_connect::Pallet::<T>::update_miner(&miner_id, &miner_type, miner);

			Self::deposit_event(Event::PaymentInitiatedForTask {
				owner: who.clone(),
				id: task_id.clone(),
				payment: details,
			});

			Self::deposit_event(Event::TaskScheduled {
				assigned_miner: miner_id,
				task_kind: submission,
				task_owner: who,
				task_id,
			});

			// TODO: Investigate gatekeeper payment management
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({<T as pallet::Config>::WeightInfo::task_scheduler_nzk(500)})]
		pub fn cancel_task(origin: OriginFor<T>, id: TaskId) -> DispatchResult {
			// check if miner is not runing the task
			let who = ensure_signed(origin)?;

			let task_info = Self::assigned_tasks(&who)
				.into_iter()
				.find(|(task_id, _)| *task_id == id)
				.map(|(_, task_info)| task_info)
				.ok_or(Error::<T>::TaskNotFound)?;

			Self::cancel(who, id.clone(), PaymentPurpose::TaskExecution(id), task_info)
		}

		#[pallet::call_index(2)]
		#[pallet::weight({<T as pallet::Config>::WeightInfo::task_scheduler_nzk(500)})]
		pub fn cancel_tasks(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get all assigned tasks for the user
			let assigned_tasks = Self::assigned_tasks(&who);

			// Cancel aLL user's task assignments using functional approach
			assigned_tasks.iter().try_for_each(|(task_id, task_info)| {
				Self::cancel(
					who.clone(),
					task_id.clone(),
					PaymentPurpose::TaskExecution(*task_id),
					task_info.clone(),
				)
			})
		}

		/// We don't modify the task storage.
		#[pallet::call_index(3)]
		#[pallet::weight({<T as pallet::Config>::WeightInfo::task_scheduler_nzk(500)})]
		pub fn terminate(origin: OriginFor<T>, id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();

			let mut task_info = Self::running_tasks(&who)
				.into_iter()
				.find(|(task_id, _)| *task_id == id)
				.map(|(_, task_info)| task_info)
				.ok_or(Error::<T>::TaskNotFound)?;

			task_info.time_elapsed = Some(now.saturating_sub(task_info.create_block));

			let Some(time_elapsed) = task_info.time_elapsed else {
				// This should never happen, but if it does, we fail fast
				return Err(Error::<T>::InvalidTaskState.into());
			};

			let refund_amount = match task_info.payment.mode {
				PaymentMode::Subscription => {
					let rate =
						T::Rate::get_rate(task_info.payment.asset.clone(), PaymentMode::OnDemand);
					let on_demand_period = <T as pallet_payment::Config>::OnDemandPeriod::get();

					let execution_cost = if time_elapsed <= on_demand_period {
						// If used less than or equal to on-demand period, pay just the on-demand
						// rate
						rate
					} else {
						// If used more than base period, pay: on_demand_rate * additional_blocks
						let mut block_count = time_elapsed.saturating_sub(on_demand_period);

						let mut total_cost = rate;

						while block_count > Zero::zero() {
							total_cost = total_cost.saturating_add(rate);
							block_count = block_count.saturating_sub(on_demand_period);
						}
						total_cost
					};
					task_info.payment.amount.saturating_sub(execution_cost)
				},
				PaymentMode::OnDemand => return Err(Error::<T>::NotAuthorized.into()),
			};

			Self::force_stop_task_and_vacate_miner(&id, task_info.clone())?;

			pallet_payment::Pallet::<T>::cashback(
				&who,
				PaymentPurpose::TaskExecution(id.clone()),
				refund_amount.clone(),
			)?;

			Self::deposit_event(Event::TaskTerminated {
				id,
				owner: who,
				refund_amount,
				execution_time: time_elapsed,
			});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_task_reception())]
		pub fn confirm_task_reception(
			origin: OriginFor<T>,
			task_id: TaskId,
			has_failed: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Load task
			let mut task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

			// If task is already running, return specific error
			if task_info.task_status == TaskStatusType::Running {
				return Err(Error::<T>::TaskReceptionAlreadyConfirmed.into());
			}

			let miner_id =
				TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::TaskNotAllocated)?;

			// Task must currently be `Assigned`
			ensure!(
				task_info.task_status == TaskStatusType::Assigned,
				Error::<T>::RequireAssignedTask
			);

			let miner_type = match task_info.task_kind {
				TaskKind::OpenInference(_) | TaskKind::FlashInfer(_) => MinerType::Edge,
				TaskKind::CyCloud(_) => MinerType::Cloud,
			};

			let miner = pallet_edge_connect::Pallet::<T>::get_miner(&miner_id, &miner_type)
				.ok_or(Error::<T>::MinerDoesNotExist)?;

			if has_failed {
				task_info.task_status = TaskStatusType::Failed;
				// TaskStatus::<T>::insert(task_id, TaskStatusType::Failed);
				Tasks::<T>::insert(task_id, task_info);

				// Update the miner status back to active
				pallet_edge_connect::Pallet::<T>::put_miner_under_maintenance(
					&miner_id,
					&miner_type,
				)?;

				Self::deposit_event(Event::TaskReceptionFailed { task_id, who });
			} else {
				ensure!(miner.owner == who, Error::<T>::NotAssignedMiner);

				let payment_details = pallet_payment::Pallet::<T>::set_active(
					&task_info.task_owner,
					PaymentPurpose::TaskExecution(task_id.clone()),
					task_info.payment.clone(),
				)?;

				// Create updated miner with new status
				let updated_miner = Miner {
					operational_status: OperationalStatus::Busy,
					status_last_updated: frame_system::Pallet::<T>::block_number(),
					..miner
				};

				pallet_edge_connect::Pallet::<T>::update_miner(
					&miner_id,
					&miner_type,
					updated_miner,
				);

				task_info.task_status = TaskStatusType::Running;
				task_info.payment = payment_details;
				//TaskStatus::<T>::insert(task_id, TaskStatusType::Running);
				Tasks::<T>::insert(task_id, task_info);

				Self::deposit_event(Event::TaskReceptionConfirmed { task_id, who });
			}

			// Remove from pending confirmations
			if let Some(assigned_block) = TaskAssignmentBlock::<T>::get(task_id) {
				let timeout_block =
					assigned_block.saturating_add(T::TaskConfirmationTimeout::get());
				PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
					if let Some(pos) = tasks.iter().position(|&id| id == task_id) {
						tasks.swap_remove(pos);
					}
				});
			}

			Ok(())
		}
		/*
		/// Signals the miner to exit task execution and reset itself
		/// Running -> Stopped
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::stop_task_and_vacate_miner())]
		pub fn stop_task_and_vacate_miner(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

			// Ensure task is owned by caller.
			ensure!(
				task.task_owner == who,
				Error::<T>::InvalidTaskOwner
			);

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
		*/

		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_gatekeeper())]
		pub fn set_gatekeeper(
			origin: OriginFor<T>,
			new_gatekeeper: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			GatekeeperAccount::<T>::put(new_gatekeeper);
			Ok(())
		}

		#[pallet::call_index(8)]
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
			let model_id_fixed: [u8; 32] =
				model_id.try_into().map_err(|_| Error::<T>::InvalidModelIdLength)?;

			// Ensure it's not already registered
			ensure!(
				!ModelHashes::<T>::contains_key(&model_id_fixed),
				Error::<T>::ModelAlreadyRegistered
			);

			// Store it
			ModelHashes::<T>::insert(&model_id_fixed, model_hash);

			// Emit event
			Self::deposit_event(Event::ModelHashRegistered(model_id_fixed.to_vec(), model_hash));
			Ok(())
		}

		// TODO: Change to view function
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::get_model_hash())]
		pub fn get_model_hash(origin: OriginFor<T>, model_id: Vec<u8>) -> DispatchResult {
			let _ = ensure_signed(origin)?; // Anyone can call

			ensure!(model_id.len() == 32, Error::<T>::InvalidModelIdLength);

			let model_id_fixed: [u8; 32] =
				model_id.try_into().map_err(|_| Error::<T>::InvalidModelIdLength)?;

			let model_hash =
				ModelHashes::<T>::get(&model_id_fixed).ok_or(Error::<T>::ModelNotFound)?;

			Self::deposit_event(Event::ModelHashQueried(model_id_fixed.to_vec(), model_hash));
			Ok(())
		}

		/// Reset a stuck task and its associated miner (sudo only)
		/// This allows manual intervention for tasks that are stuck in Assigned, Running, or
		/// Stopped states
		#[pallet::call_index(10)]
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
			let task_allocation =
				TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::TaskNotAllocated)?;

			// Store the assigned block
			let assigned_block = TaskAssignmentBlock::<T>::get(task_id);

			// Reset miner status
			Self::reset_miner_for_task(&task_allocation, miner_type.clone(), &task_id)?;

			// Clean up task storage
			Tasks::<T>::remove(task_id);
			TaskAllocations::<T>::remove(task_id);
			TaskAssignmentBlock::<T>::remove(task_id);

			// Remove from pending confirmations if present
			if let Some(assigned_block) = assigned_block {
				let timeout_block =
					assigned_block.saturating_add(T::TaskConfirmationTimeout::get());

				PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
					if let Some(pos) = tasks.iter().position(|&id| id == task_id) {
						tasks.swap_remove(pos);
					}
				});
			}

			Self::deposit_event(Event::TaskManuallyReset {
				id: task_id,
				reset_by: None,
				previous_status,
				reason,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check for expired task confirmations and penalize miners
		pub fn check_task_confirmation_timeouts() -> DispatchResult {
			let now = frame_system::Pallet::<T>::block_number();

			// Only check tasks that are due at or before current_block
			for (timeout_block, task_ids) in PendingTaskConfirmations::<T>::iter() {
				if timeout_block > now {
					break;
				}

				for task_id in task_ids.iter() {
					if let Some(task_info) = Tasks::<T>::get(task_id) {
						if task_info.task_status != TaskStatusType::Assigned {
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
							TaskAllocations::<T>::remove(task_id);
							TaskAssignmentBlock::<T>::remove(task_id);
						}
					}
				}

				// Remove processed block from storage
				PendingTaskConfirmations::<T>::remove(timeout_block);
			}

			Ok(())
		}

		fn reset_miner_for_task(
			miner_key: &MinerId,
			miner_type: MinerType,
			task_id: &TaskId,
		) -> DispatchResult {
			// Get current miner state
			let existing_miner =
				pallet_edge_connect::Pallet::<T>::get_miner(miner_key, &miner_type)
					.ok_or(Error::<T>::MinerResetFailed)?;

			// Only reset if the miner is currently working on this task
			if let Some(current_task) = existing_miner.current_task {
				if current_task == *task_id {
					// Create updated miner with new status
					let miner = Miner {
						operational_status: OperationalStatus::Available,
						status_last_updated: frame_system::Pallet::<T>::block_number(),
						current_task: None,
						..existing_miner
					};
					// Reset miner to available status
					pallet_edge_connect::Pallet::<T>::update_miner(
						miner_key,
						&miner_type,
						miner.clone(),
					);

					// If miner was suspended due to this task, lift suspension
					if miner.is_suspended() {
						let _ = pallet_edge_connect::Pallet::<T>::lift_suspension(
							miner_key,
							&miner_type,
						);
					}
				}
			}

			Ok(())
		}

		pub fn force_stop_task_and_vacate_miner(
			task_id: &TaskId,
			task_info: TaskInfoOf<T>,
		) -> DispatchResult {
			// Get assigned miner before cleaning up task storage
			let miner_id =
				TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::TaskNotAllocated)?;
			let miner_type = match task_info.task_kind {
				TaskKind::OpenInference(_) | TaskKind::FlashInfer(_) => MinerType::Edge,
				TaskKind::CyCloud(_) => MinerType::Cloud,
			};

			Self::cleanup(task_id.clone());

			// Get the existing miner first, then update it
			let existing_miner =
				pallet_edge_connect::Pallet::<T>::get_miner(&miner_id, &miner_type)
					.ok_or(Error::<T>::MinerDoesNotExist)?;

			// Create updated miner with new status
			let miner = Miner {
				operational_status: OperationalStatus::Available,
				status_last_updated: frame_system::Pallet::<T>::block_number(),
				current_task: None,
				..existing_miner
			};

			pallet_edge_connect::Pallet::<T>::update_miner(&miner_id, &miner_type, miner);

			Ok(())
		}

		fn stop_tasks_for_expired_payments() -> DispatchResult {
			let stopped_tasks: Vec<StoppedTaskInfoOf<T>> = Tasks::<T>::iter()
				.filter(|(_, task_info)| task_info.task_status == TaskStatusType::Running)
				.filter_map(|(task_id, task_info)| {
					let active = pallet_payment::Pallet::<T>::has_active_payment(
						&task_info.task_owner,
						PaymentPurpose::TaskExecution(task_id),
					);

					if !active {
						// Try to stop the task and get the stopped task info
						match Self::stop_task_and_vacate_miner(task_id, task_info) {
							Ok(stopped_task_info) => Some(stopped_task_info),
							Err(e) => {
								log::error!("Failed to stop task {}: {:?}", task_id, e);
								None
							},
						}
					} else {
						None
					}
				})
				.collect();

			let stopped_count = stopped_tasks.len() as u32;

			if stopped_count > 0u32 {
				log::info!("Stopped {} tasks due to expired payments", stopped_count);
				Self::deposit_event(Event::TasksStoppedForExpiredPayments {
					count: stopped_count,
					stopped_tasks,
				});
			}
			Ok(())
		}

		fn stop_task_and_vacate_miner(
			task_id: TaskId,
			info: TaskInfoOf<T>,
		) -> Result<StoppedTaskInfoOf<T>, DispatchError> {
			// Get assigned miner and update their status
			if let Some(assigned_miner) = TaskAllocations::<T>::get(task_id) {
				// Determine miner type from task kind
				let miner_type = match info.task_kind {
					TaskKind::OpenInference(_) | TaskKind::FlashInfer(_) => MinerType::Edge,
					TaskKind::CyCloud(_) => MinerType::Cloud,
				};

				// Get the existing miner first, then update it
				let existing_miner =
					pallet_edge_connect::Pallet::<T>::get_miner(&assigned_miner, &miner_type)
						.ok_or(Error::<T>::MinerDoesNotExist)?;

				// Create updated miner
				let miner = Miner {
					operational_status: OperationalStatus::Available,
					status_last_updated: frame_system::Pallet::<T>::block_number(),
					current_task: None,
					..existing_miner
				};

				pallet_edge_connect::Pallet::<T>::update_miner(&assigned_miner, &miner_type, miner);
				log::info!("Vacated miner for task {} due to payment expiration", task_id);
			}
			Self::cleanup(task_id);
			Ok((task_id, info.payment))
		}

		/// Check if user has any assigned tasks (using the iter approach)
		fn assigned_tasks(who: &T::AccountId) -> Vec<(TaskId, TaskInfoOf<T>)> {
			Tasks::<T>::iter()
				.filter(|(_, task_info)| {
					task_info.task_owner == *who &&
						task_info.task_status == TaskStatusType::Assigned
				})
				.collect()
		}

		fn running_tasks(who: &T::AccountId) -> Vec<(TaskId, TaskInfoOf<T>)> {
			Tasks::<T>::iter()
				.filter(|(_, task_info)| {
					task_info.task_owner == *who && task_info.task_status == TaskStatusType::Running
				})
				.collect()
		}

		fn cancel(
			owner: T::AccountId,
			id: TaskId,
			purpose: PaymentPurpose,
			info: TaskInfoOf<T>,
		) -> DispatchResult {
			// Return payment execution reserve.
			pallet_payment::Pallet::<T>::release(&owner, purpose)?;

			// Get assigned miner
			let miner_id = TaskAllocations::<T>::get(id).ok_or(Error::<T>::TaskNotAllocated)?;

			let miner_type = match info.task_kind {
				TaskKind::OpenInference(_) | TaskKind::FlashInfer(_) => MinerType::Edge,
				TaskKind::CyCloud(_) => MinerType::Cloud,
			};

			Self::cleanup(id.clone());

			// Get the existing miner first, then update it
			let existing_miner =
				pallet_edge_connect::Pallet::<T>::get_miner(&miner_id, &miner_type)
					.ok_or(Error::<T>::MinerDoesNotExist)?;

			// Create updated miner with new status
			let miner = Miner {
				operational_status: OperationalStatus::Available,
				status_last_updated: frame_system::Pallet::<T>::block_number(),
				current_task: None,
				..existing_miner
			};

			pallet_edge_connect::Pallet::<T>::update_miner(&miner_id, &miner_type, miner);

			Self::deposit_event(Event::TaskCancelled { id, owner });

			Ok(())
		}

		/// Clean up all task-related storage, only applies to taskassignment before confirm
		fn cleanup(task_id: TaskId) {
			// Remove miner assignment
			TaskAllocations::<T>::remove(task_id);
			// Remove task data
			Tasks::<T>::remove(task_id);

			// TODO: When task is running or has stopped we do not need to do the below stuff.
			if let Some(assigned_at) = TaskAssignmentBlock::<T>::get(task_id) {
				let timeout_block = assigned_at.saturating_add(T::TaskConfirmationTimeout::get());
				PendingTaskConfirmations::<T>::mutate(timeout_block, |tasks| {
					if let Some(pos) = tasks.iter().position(|&id| id == task_id) {
						tasks.swap_remove(pos);
					}
				});
				TaskAssignmentBlock::<T>::remove(task_id);
			}
		}
	}

	impl<T: Config + timestamp::Config>
		NzkTaskInfoHandler<T::AccountId, TaskId, BlockNumberFor<T>, AssetIdOf<T>, BalanceOf<T>>
		for Pallet<T>
	{
		// Implementation of the NzkTaskInfoHandler trait, which provides methods for accessing NZK
		// task information.
		fn get_nzk_task(task_key: TaskId) -> Option<TaskInfoOf<T>> {
			Tasks::<T>::get(task_key)
		}

		// Implementation of the NzkTaskInfoHandler trait, which provides methods for NZK task
		// information.
		fn update_nzk_task(task_key: TaskId, task: TaskInfoOf<T>) {
			Tasks::<T>::insert(task_key, task);
		}
	}
}
