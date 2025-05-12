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

pub use cyborg_primitives::task::*;
use cyborg_primitives::worker::WorkerId;
use cyborg_primitives::worker::WorkerType;
use pallet_edge_connect::{ExecutableWorkers, WorkerClusters};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::PostDispatchInfo;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::{OriginFor, *};
	// use pallet_edge_connect::AccountWorkers;

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

	/// Pallets use events to inform users when important changes are made.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new task has been scheduled and assigned to a worker.
		TaskScheduled {
			assigned_worker: (T::AccountId, WorkerId),
			task_kind: TaskKind,
			task_owner: T::AccountId,
			task_id: TaskId,
			task: BoundedVec<u8, ConstU32<500>>,
			zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
		},

		/// A worker confirmed reception of task data and started execution.
		TaskReceptionConfirmed { task_id: TaskId, who: T::AccountId },

		/// Controller/admin requested to stop a running task.
		TaskStopRequested { task_id: TaskId },

		/// Miner confirmed that they have vacated/reset after stopping.
		MinerVacated { task_id: TaskId },
	}

	/// Errors inform users that something went wrong.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::error]
	pub enum Error<T> {
		InvalidTaskState,
		NotTaskOwner,
		UnexpectedZkFiles,
		TaskNotFound,
		// Scheduling errors
		RequireComputeHoursDeposit, // A compute hour deposit is required to schedule or proceed with the task.
		ZkFilesMissing, // The user submitted a ZK task, but has not provided the required files for proof generation
		NoWorkersAvailable, // No workers are available for the task.

		// General task errors
		UnassignedTaskId,         // The provided task ID does not exist.
		InvalidTaskOwner,         // The caller is not the task owner.
		TaskVerificationNotFound, // The task verification process cannot be found.

		// Status transition errors
		RequireAssignedTask, // A task must be assigned before it can proceed to the next step.

		// Verification-specific errors
		RequireAssignedVerifier, // A verifier must be assigned to the task.

		/// Account has exceeded task submission rate limit
		RateLimitExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new task and assigns it to a randomly selected worker.
		/// None -> Assigned
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::task_scheduler(task_data.len() as u32))]
		pub fn task_scheduler(
			origin: OriginFor<T>,
			task_kind: TaskKind,
			task_data: BoundedVec<u8, ConstU32<500>>,
			zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			compute_hours_deposit: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check rate limit
			Self::check_rate_limit(&who)?;

			// Determine worker type based on task kind
			let worker_type = match task_kind {
				TaskKind::NeuroZK => WorkerType::Executable,
				TaskKind::OpenInference => WorkerType::Docker,
			};

			// Check worker status and reputation
			pallet_edge_connect::Pallet::<T>::check_worker_status(
				&(worker_owner.clone(), worker_id),
				worker_type,
			)?;

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

			// Check task_type and task_kind compatibility
			match task_kind {
				TaskKind::NeuroZK => {
					// For NeuroZK, zk_files_cid must be present
					ensure!(zk_files_cid.is_some(), Error::<T>::ZkFilesMissing);
				}
				TaskKind::OpenInference => {
					// zk_files_cid should not be present for normal inference
					ensure!(zk_files_cid.is_none(), Error::<T>::UnexpectedZkFiles);
				}
			}

			// First check if any workers exist at all
			let any_workers_exist = match task_kind {
				TaskKind::NeuroZK => !ExecutableWorkers::<T>::iter().next().is_none(),
				TaskKind::OpenInference => !WorkerClusters::<T>::iter().next().is_none(),
			};
			ensure!(any_workers_exist, Error::<T>::NoWorkersAvailable);

			// Ensure the specific worker exists in the correct storage
			let worker_exists = match task_kind {
				TaskKind::NeuroZK => pallet_edge_connect::ExecutableWorkers::<T>::contains_key((
					worker_owner.clone(),
					worker_id,
				)),
				TaskKind::OpenInference => {
					pallet_edge_connect::WorkerClusters::<T>::contains_key((worker_owner.clone(), worker_id))
				}
			};
			ensure!(worker_exists, Error::<T>::NoWorkersAvailable);

			// Consume compute hours from payment pallet
			pallet_payment::Pallet::<T>::consume_compute_hours(origin.clone(), deposit)?;

			// Generate task ID
			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.wrapping_add(1));

			let selected_worker = (worker_owner, worker_id);

			let task_info = TaskInfo::<T::AccountId, BlockNumberFor<T>> {
				task_owner: who.clone(),
				create_block: <frame_system::Pallet<T>>::block_number(),
				metadata: task_data.clone(),
				zk_files_cid: zk_files_cid.clone(),
				time_elapsed: None,
				average_cpu_percentage_use: None,
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
				task_kind,
				task_owner: who,
				task_id,
				task: task_data,
				zk_files_cid,
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
			let mut task_info = Tasks::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;

			// Check that caller is the assigned worker
			let assigned_worker =
				TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
			ensure!(assigned_worker.0 == who, Error::<T>::InvalidTaskOwner);

			// Task must currently be `Assigned`
			ensure!(
				task_info.task_status == TaskStatusType::Assigned,
				Error::<T>::RequireAssignedTask
			);

			// Transition task to `Running`
			task_info.task_status = TaskStatusType::Running;
			TaskStatus::<T>::insert(task_id, TaskStatusType::Running);

			// Store back the updated task info
			Tasks::<T>::insert(task_id, task_info);

			// Start compute aggregation: record starting block
			ComputeAggregations::<T>::insert(
				task_id,
				(
					<frame_system::Pallet<T>>::block_number(),
					None::<BlockNumberFor<T>>,
				),
			);

			// Emit event (you can define a new event like TaskReceptionConfirmed if needed)
			Self::deposit_event(Event::TaskReceptionConfirmed { task_id, who });

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
		pub fn confirm_miner_vacation(origin: OriginFor<T>, task_id: TaskId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

			// Ensure task owner is confirming.
			ensure!(task.task_owner == who, Error::<T>::NotTaskOwner);

			// Ensure task is stopped.
			ensure!(
				task.task_status == TaskStatusType::Stopped,
				Error::<T>::InvalidTaskState
			);

			// Move to Vacated state.
			task.task_status = TaskStatusType::Vacated;
			Tasks::<T>::insert(task_id, task);

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
	}

	impl<T: Config> Pallet<T> {
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
	}
}
