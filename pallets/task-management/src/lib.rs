#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub use pallet_worker_clusters;
use scale_info::{ TypeInfo };
use frame_support::{sp_runtime::RuntimeDebug, BoundedVec, pallet_prelude::ConstU32 };
use codec::{ Encode, Decode, MaxEncodedLen };
use sp_core::hash::H256;

pub type TaskId = u64;

use pallet_worker_clusters::types::{WorkerId, WorkerStatusType};

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum TaskStatusType {
	Assigned,
	PendingValidation,
	Completed,
	Expired,
}

#[derive(PartialEq, Eq, Clone, Decode, Encode, TypeInfo, Debug, MaxEncodedLen)]
pub enum TaskType {
	Docker,
}


#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TaskInfo<AccountId, BlockNumber> {
	pub task_owner: AccountId,
	pub create_block: BlockNumber,
	pub metadata: BoundedVec<u8, ConstU32<128>>,
	pub time_elapsed: Option<BlockNumber>,
	pub average_cpu_percentage_use: Option<u8>,
	pub task_type: TaskType,
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerificationHashes<AccountId> {
	pub account: AccountId,
	pub completed_hash: Option<H256>
}

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Verifications<AccountId> {
	executor: VerificationHashes<AccountId>,
	verifier: Option<VerificationHashes<AccountId>>,
	resolver: Option<VerificationHashes<AccountId>>,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::{WeightInfo, pallet_prelude::*};
	use super::*;
	use pallet_worker_clusters::{Pallet as PalletWorkerClusters, AccountWorkers, WorkerClusters};
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_worker_clusters::Config {
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
    pub type TaskAllocations<T: Config> = StorageMap<_, Twox64Concat, TaskId, (T::AccountId, WorkerId), OptionQuery>;

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
    type TaskVerifications<T: Config> = StorageMap<_, Blake2_128Concat, TaskId, Verifications<T::AccountId>, OptionQuery>;


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
		SubmittedCompletedTask{ task_id: TaskId, assigned_verifier: (T::AccountId, WorkerId) },
		VerifierResolverAssigned{ task_id: TaskId, assigned_resolver: (T::AccountId, WorkerId) },
		VerifiedCompletedTask{ task_id: TaskId },
	}

	/// Errors inform users that something went wrong.
	/// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
	#[pallet::error]
	pub enum Error<T> {
		UnassignedTaskId,
		InvalidTaskOwner,
		RequireAssignedTask,
		RequireAssignedVerifier,
		RequireAssignedResolver,
		NoWorkersAvailable,
		TaskVerificationNotFound,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Creates a task and assigns it to an available worker
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]		
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
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn submit_completed_task(
			origin: OriginFor<T>,
			task_id: TaskId,
			completed_hash: H256,
		) -> DispatchResult
		{
			let who = ensure_signed(origin)?;

			let task_assignee = TaskAllocations::<T>::get(task_id).ok_or(Error::<T>::UnassignedTaskId)?;
			ensure!(task_assignee.0 == who, Error::<T>::InvalidTaskOwner);
			ensure!(TaskStatus::<T>::get(task_id) == Some(TaskStatusType::Assigned), Error::<T>::RequireAssignedTask);

			// update storage info
			TaskStatus::<T>::insert(task_id, TaskStatusType::PendingValidation);

			let mut ver = Verifications {
				executor: VerificationHashes {
					account: who.clone(), completed_hash: Some(completed_hash),
				},
				verifier: None,
				resolver: None,
			};

			let workers: Vec<_> = WorkerClusters::<T>::iter()
			.filter(|&(_, ref worker)| worker.status == WorkerStatusType::Active && worker.owner != who.clone())
			.collect::<Vec<_>>();

			let random_index = (sp_io::hashing::blake2_256(&ver.encode())[0] as usize) % workers.len();
			let assigned_verifier: (T::AccountId, WorkerId) = workers[random_index].0.clone();

			ver.verifier = Some(VerificationHashes {
				account: assigned_verifier.0.clone(), completed_hash: None,
			});

			TaskVerifications::<T>::insert(task_id, ver.clone());
			// Emit an event.
			Self::deposit_event(Event::SubmittedCompletedTask { task_id, assigned_verifier });
			Ok(())
		} 

		/// Verifies completed task once a threshold of verifiers have succussfully validate correct completed task
		/// Can only be called from root
		/// Assign new verifier as resolver if verification does not match. Resolver will determine correct result.
		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn verify_completed_task(
			origin: OriginFor<T>,
			account: T::AccountId,
			task_id: TaskId,
			completed_hash: H256,
		) -> DispatchResult
		{
			let who = ensure_root(origin)?;
			ensure!(TaskStatus::<T>::get(task_id) == Some(TaskStatusType::PendingValidation), Error::<T>::RequireAssignedTask);
			let taskVerification = TaskVerifications::<T>::get(task_id);
			// ensure!(taskVerification.is_some(), Error::<T>::TaskVerificationNotFound);
		
			// check completed hashes are the same
			match taskVerification {
                Some(ref verification) => {
					ensure!(verification.verifier.as_ref().map_or(false, |v| v.account == account), Error::<T>::RequireAssignedVerifier);
                    // Hashes Match
					if verification.executor.completed_hash == Some(completed_hash){
						TaskStatus::<T>::insert(task_id, TaskStatusType::Completed);
						// Emit an event.
						Self::deposit_event(Event::VerifiedCompletedTask { task_id });
					} else {
						// Assign new verifier as resolver if verification does not match
						let mut new_verification = verification.clone();
						
						// Find available workers that are not the executor or current verifier
						let workers: Vec<_> = WorkerClusters::<T>::iter()
						.filter(|&(_, ref worker)| worker.status == WorkerStatusType::Active 
							&& verification.verifier.as_ref().map_or(false, |v| v.account != worker.owner)
							&& worker.owner != verification.executor.account.clone())
						.collect::<Vec<_>>();

						// .extend(&<frame_system::Pallet<T>>::block_number().encode())
						let random_index = (sp_io::hashing::blake2_256(&taskVerification.encode())[0] as usize) % workers.len();
						let assigned_resolver: (T::AccountId, WorkerId) = workers[random_index].0.clone();

						new_verification.resolver = Some(VerificationHashes {
							account: assigned_resolver.0.clone(), completed_hash: None,
						});
						TaskVerifications::<T>::insert(task_id, new_verification);

						// Emit an event.
						Self::deposit_event(Event::VerifierResolverAssigned { task_id, assigned_resolver });
					};
                },
                None => {
					return Err(Error::<T>::TaskVerificationNotFound.into());
				}
            };
			Ok(())
		} 
	}
}
