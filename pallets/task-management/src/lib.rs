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
pub use cyborg_primitives::task::*;
use cyborg_primitives::worker::WorkerId;
use cyborg_primitives::worker::WorkerType;
use pallet_edge_connect::ExecutableWorkers;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::{DispatchResultWithPostInfo, PostDispatchInfo};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::{OriginFor, *};
	use pallet_timestamp as timestamp;
	use sp_core::ByteArray;
	use frame_support::sp_runtime::SaturatedConversion;
	// use pallet_edge_connect::AccountWorkers;

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
	pub type GatekeeperAccount<T: Config> = StorageValue<_, sp_core::sr25519::Public, OptionQuery>;

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
	pub type ModelHashes<T: Config> = StorageMap<
		  _, 
		Blake2_128Concat, 
		[u8; 32],     
		T::Hash, 
		OptionQuery
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
		},

		/// A worker confirmed reception of task data and started execution.
		TaskReceptionConfirmed { task_id: TaskId, who: T::AccountId },

		/// Controller/admin requested to stop a running task.
		TaskStopRequested { task_id: TaskId },

		/// Miner confirmed that they have vacated/reset after stopping.
		MinerVacated { task_id: TaskId },
		ModelHashRegistered(Vec<u8>, T::Hash),
		ModelHashQueried(Vec<u8>, T::Hash),


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
		TaskNotFound,
		InvalidModelId,
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
		WorkerDoesNotExist,
		ModelAlreadyRegistered,
		ModelNotFound,
		/// The signature of the gatekeeper is invalid
		InvalidGatekeeperSignature,
		/// The wrong nonce was signed by the gatekeeper
		GatekeeperSignatureUserNonceMismatch,
		/// The signature of the gatekeeper has the wrong length
		InvalidGatekeeperSignatureLength,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new task and assigns it to a randomly selected worker.
		/// None -> Assigned
		#[pallet::call_index(0)]
		#[pallet::weight({
    		if nzk_info.is_some() {
        		<T as pallet::Config>::WeightInfo::task_scheduler_nzk(task_location.len() as u32)
    		} else {
        		<T as pallet::Config>::WeightInfo::task_scheduler_no_nzk(task_location.len() as u32)
    		}
		})]
		pub fn task_scheduler(
			origin: OriginFor<T>,
			gatekeeper_message: Option<SignedGatekeeperMessage<T::AccountId>>,
			task_kind: TaskKind,
			task_location: BoundedVec<u8, ConstU32<500>>,
			nzk_info: Option<NeuroZkTaskSubmissionDetails>,
			worker_owner: T::AccountId,
			worker_id: WorkerId,
			compute_hours_deposit: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Determine worker type based on task kind
			let worker_type = match task_kind {
				TaskKind::NeuroZK => WorkerType::Executable,
				TaskKind::OpenInference => WorkerType::Executable,
			};

			// Check if any workers exist for the task_kind first
			let any_workers_exist = match task_kind {
				TaskKind::NeuroZK => ExecutableWorkers::<T>::iter().next().is_some(),
				TaskKind::OpenInference => ExecutableWorkers::<T>::iter().next().is_some(),
			};
			ensure!(any_workers_exist, Error::<T>::NoWorkersAvailable);

			// Check worker status and reputation
			pallet_edge_connect::Pallet::<T>::check_worker_status(
				&(worker_owner.clone(), worker_id),
				worker_type,
			)
			.map_err(|_| Error::<T>::WorkerDoesNotExist)?;

			// Then check if the specific worker exists
			let worker_exists = match task_kind {
				TaskKind::NeuroZK => {
					ExecutableWorkers::<T>::contains_key((worker_owner.clone(), worker_id))
				}
				TaskKind::OpenInference => {
					ExecutableWorkers::<T>::contains_key((worker_owner.clone(), worker_id))
				}
			};
			ensure!(worker_exists, Error::<T>::WorkerDoesNotExist);

			let is_gatekeeper_valid = Self::validate_gatekeeper_sig(&gatekeeper_message, &who)?;
			let pays_fee = if is_gatekeeper_valid {
    			Pays::No
			} else {
    			Pays::Yes
			};

			// Validate deposit
			let deposit = compute_hours_deposit.ok_or(Error::<T>::RequireComputeHoursDeposit)?;
			ensure!(deposit > 0, Error::<T>::RequireComputeHoursDeposit);

			let mut nzk_data = None;

			match task_kind {
				TaskKind::NeuroZK => {
					// For NeuroZK, zk_info must be present
					match nzk_info {
						Some(data) => {
							nzk_data = Some(NzkData {
								zk_input: data.zk_input,
								zk_settings: data.zk_settings,
								zk_verifying_key: data.zk_verifying_key,
								zk_proof: None,
								last_proof_accepted: None,
							});
						}
						None => {
							return Err(Error::<T>::ZkFilesMissing.into());
						}
					}
				}
				TaskKind::OpenInference => {
					ensure!(nzk_info.is_none(), Error::<T>::UnexpectedZkFiles);
				}
			}

			// Consume compute hours from payment pallet
			pallet_payment::Pallet::<T>::consume_compute_hours(origin.clone(), deposit)?;

			// Generate task ID
			let task_id = NextTaskId::<T>::get();
			NextTaskId::<T>::put(task_id.wrapping_add(1));

			let selected_worker = (worker_owner, worker_id);

			let task_info = TaskInfo::<T::AccountId, BlockNumberFor<T>> {
				task_owner: who.clone(),
				create_block: <frame_system::Pallet<T>>::block_number(),
				metadata: task_location.clone(),
				nzk_data,
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
				task: task_location,
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
			new_gatekeeper: sp_core::sr25519::Public,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			GatekeeperAccount::<T>::put(new_gatekeeper);
			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_model_hash())]
		pub fn register_model_hash(
			origin: OriginFor<T>,
			gatekeeper_message: Option<SignedGatekeeperMessage<T::AccountId>>,
			model_id: Vec<u8>,
			model_hash: T::Hash,
		) -> DispatchResultWithPostInfo {
		let who = ensure_signed(origin)?;

		let is_gatekeeper_valid = Self::validate_gatekeeper_sig(&gatekeeper_message, &who)?;
		let pays_fee = if is_gatekeeper_valid {
    		Pays::No
		} else {
			return Err(Error::<T>::InvalidGatekeeperSignature.into());
		};

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
		Self::deposit_event(Event::ModelHashRegistered(model_id_fixed.to_vec(), model_hash));

		Ok(PostDispatchInfo {
			actual_weight: None,
			pays_fee,
		})
	}

	#[pallet::call_index(8)]
	#[pallet::weight(<T as pallet::Config>::WeightInfo::get_model_hash())]
	pub fn get_model_hash(
		origin: OriginFor<T>,
		model_id: Vec<u8>,
	) -> DispatchResult {
		let _ = ensure_signed(origin)?; // Anyone can call

		ensure!(model_id.len() == 32, Error::<T>::InvalidModelIdLength);

		let model_id_fixed: [u8; 32] = model_id
			.try_into()
			.map_err(|_| Error::<T>::InvalidModelIdLength)?;

		let model_hash = ModelHashes::<T>::get(&model_id_fixed)
			.ok_or(Error::<T>::ModelNotFound)?;

		Self::deposit_event(Event::ModelHashQueried(model_id_fixed.to_vec(), model_hash));
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

		fn validate_gatekeeper_sig(maybe_sig: &Option<SignedGatekeeperMessage<T::AccountId>>, who: &T::AccountId) -> Result<bool, DispatchError> {
			if let Some(signed_message) = maybe_sig {
				let gatekeeper_key = GatekeeperAccount::<T>::get()
					.ok_or(Error::<T>::NotGatekeeper)?;
				let signature = sp_core::sr25519::Signature::from_slice(&signed_message.signature)
					.map_err(|_| Error::<T>::InvalidGatekeeperSignatureLength)?;

				let is_valid = sp_io::crypto::sr25519_verify(
					&signature,
					&signed_message.message_nonce.to_le_bytes(),
					&gatekeeper_key,
				);

				ensure!(is_valid, Error::<T>::InvalidGatekeeperSignature);

				let account_nonce = frame_system::Pallet::<T>::account(who).nonce.saturated_into::<u64>();

				ensure!(
					account_nonce == signed_message.message_nonce,
					Error::<T>::GatekeeperSignatureUserNonceMismatch
				);

				return Ok(true);
			}

			Ok(false)
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
