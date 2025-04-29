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

use frame_support::{pallet_prelude::*, storage::bounded_vec::BoundedVec};
use frame_system::pallet_prelude::*;
use scale_info::prelude::vec::Vec;
use sp_core::H256;

/// A convenient alias for task ID type.
type TaskId = u64;



/// Enum representing the current state of the task.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum TaskState {
    Scheduled,
    Running,
    Stopped,
    Vacated,
}

/// Trait for verifying ZK tasks.
/// This allows loose coupling between task manager and neurozk-verifier pallet.
pub trait ZKVerifier<AccountId> {
    fn setup_verification(task_id: TaskId, public_inputs: Vec<u8>, verification_key: Vec<u8>) -> DispatchResult;
    fn verify_proof(who: AccountId, task_id: TaskId, proof: Vec<u8>) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Main pallet declaration.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet configuration trait.
    /// Specifies which external types and constants this pallet depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight info for benchmarking.
        type WeightInfo: WeightInfo;

        /// Interface to the NeuroZK Verifier.
        type NeuroZKVerifier: ZKVerifier<Self::AccountId>;
    }

    /// Storage map for keeping track of tasks and their metadata.
    #[pallet::storage]
    #[pallet::getter(fn tasks)]
    pub type Tasks<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        TaskId, 
        TaskInfo<T::AccountId, BlockNumberFor<T>>, 
        OptionQuery
    >;

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

    /// Storage for keeping track of the next available task ID.
    #[pallet::storage]
    #[pallet::getter(fn next_task_id)]
    pub type NextTaskId<T: Config> = StorageValue<_, TaskId, ValueQuery>;

    /// Struct holding detailed information about a task.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct TaskInfo<AccountId, BlockNumber> {
        pub task_kind: TaskKind,
        pub task_owner: AccountId,
        pub model_location: Option<BoundedVec<u8, ConstU32<500>>>,
        pub encryption_cypher: Option<BoundedVec<u8, ConstU32<500>>>,
        pub zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
        pub task_state: TaskState,
        pub created_at: BlockNumber,
    }

    /// Events emitted by the pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A task was scheduled.
        TaskScheduled { task_id: TaskId, who: T::AccountId },

        /// A miner confirmed task reception and started execution.
        TaskReceptionConfirmed { task_id: TaskId, who: T::AccountId },

        /// Controller asked miner to stop a task.
        TaskStopRequested { task_id: TaskId },

        /// Miner confirmed it has vacated/reset after stopping the task.
        MinerVacated { task_id: TaskId },
    }

    /// Errors that the pallet might throw.
    #[pallet::error]
    pub enum Error<T> {
        /// Task ID not found.
        TaskNotFound,

        /// Task is in an invalid state for the requested operation.
        InvalidTaskState,

        /// Only task owner can perform the action.
        NotTaskOwner,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Schedule a new task to be executed by a miner.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::schedule_task())]
        pub fn schedule_task(
            origin: OriginFor<T>,
            task_kind: TaskKind,
            model_location: Option<BoundedVec<u8, ConstU32<500>>>,
            encryption_cypher: Option<BoundedVec<u8, ConstU32<500>>>,
            zk_files_cid: Option<BoundedVec<u8, ConstU32<500>>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get the next available task ID.
            let task_id = NextTaskId::<T>::get();
            NextTaskId::<T>::put(task_id + 1);

            // Create a task info structure.
            let task_info = TaskInfo {
                task_kind: task_kind.clone(),
                task_owner: who.clone(),
                model_location,
                encryption_cypher,
                zk_files_cid,
                task_state: TaskState::Scheduled,
                created_at: <frame_system::Pallet<T>>::block_number(),
            };

            // Insert the new task into storage.
            Tasks::<T>::insert(task_id, task_info);

            // Emit event.
            Self::deposit_event(Event::TaskScheduled { task_id, who });

            Ok(())
        }

        /// Miner confirms that they have received the task and started working on it.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::confirm_task_reception())]
        pub fn confirm_task_reception(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure task exists.
            let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

            // Only the task owner (miner) can confirm reception.
            ensure!(task.task_owner == who, Error::<T>::NotTaskOwner);

            // Ensure that the task is still scheduled.
            ensure!(task.task_state == TaskState::Scheduled, Error::<T>::InvalidTaskState);

            // Move task state to Running.
            task.task_state = TaskState::Running;
            Tasks::<T>::insert(task_id, task);

            // Start compute aggregation.
            ComputeAggregations::<T>::insert(task_id, (<frame_system::Pallet<T>>::block_number(), None));

            // Emit event.
            Self::deposit_event(Event::TaskReceptionConfirmed { task_id, who });

            Ok(())
        }

        /// Controller stops the task execution and asks miner to vacate.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::stop_task_and_vacate_miner())]
        pub fn stop_task_and_vacate_miner(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            ensure_signed(origin)?; // anyone controlling can request stop

            let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

            // Ensure task is running.
            ensure!(task.task_state == TaskState::Running, Error::<T>::InvalidTaskState);

            // Change task state to Stopped.
            task.task_state = TaskState::Stopped;
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

        /// Miner confirms that they have vacated and reset after stopping.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::confirm_miner_vacation())]
        pub fn confirm_miner_vacation(
            origin: OriginFor<T>,
            task_id: TaskId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

            // Ensure task owner is confirming.
            ensure!(task.task_owner == who, Error::<T>::NotTaskOwner);

            // Ensure task is stopped.
            ensure!(task.task_state == TaskState::Stopped, Error::<T>::InvalidTaskState);

            // Move to Vacated state.
            task.task_state = TaskState::Vacated;
            Tasks::<T>::insert(task_id, task);

            // Emit event.
            Self::deposit_event(Event::MinerVacated { task_id });

            Ok(())
        }
    }
}
