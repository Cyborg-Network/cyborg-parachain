#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{traits::Get, BoundedVec, ensure};
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::offchain::{http, Duration};
use sp_runtime::sp_std::{vec};

pub use cyborg_primitives::{
	proof::*,
	task::{TaskId, TaskType},
};
use codec::{Decode, Encode, WrapperTypeEncode, WrapperTypeDecode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_runtime::DispatchResult;

//#[cfg(test)]
//mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"n-zk");

pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	use codec::{Decode, Encode, WrapperTypeEncode, WrapperTypeDecode, EncodeLike, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_runtime::RuntimeDebug;
	app_crypto!(sr25519, KEY_TYPE);

	#[derive(
		Encode,
		Decode,
		MaxEncodedLen,
		TypeInfo,
		Clone,
		PartialEq,
		Eq,
		RuntimeDebug,
	)]			
	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>> + frame_system::Config
	{
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature> + Encode + Decode + EncodeLike + MaxEncodedLen + TypeInfo;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		//TODO: This might be unecessary or even hindering
		/// Number of blocks to wait before ocw submits result
		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;

		/// Max number of proof verifiers, this is how many accounts will be assigned for verification
		#[pallet::constant]
		type MaxNumOfProofVerifiers: Get<u32>;

		/// Of the max `MaxNumOfVerifiers` there might be verifiers that are offline, so if the
		/// `MinNumberOfVerifiers` isn't reached, the verification process will be repeated
		#[pallet::constant]
		type MinNumOfProofVerifiers: Get<u32>;

		/// Number of blocks to wait for ocws to submit their results
	    #[pallet::constant]
    	type FinalizationDelay: Get<BlockNumberFor<Self>>;

		/// Max number of scheduled tasks for verification
    	#[pallet::constant]
    	type MaxScheduledFinalizations: Get<u32>;	

		/// Required amount of token to stake for pariticipation in verification
		#[pallet::constant]
		type Stake: Get<u32>;

		/// The maximum amount of tasks that can be in the queue
		#[pallet::constant]
		type MaxTasks: Get<u32>;

		/// The maximum amount of tasks that can be verified per block
		#[pallet::constant]
		type MaxTasksPerBlock: Get<u32>;
	}

	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when the user requests a proof for a certain task
		ProofRequested { task_id: TaskId },
		/// Event generated when new results are submitted by one offchain worker
		NewResult {
			results: NodeProofResponse<T::MaxTasksPerBlock>,
			who: T::AccountId,
		},
		/// Event generated when the proof has successfully been verified
		ProofVerified { task_id: TaskId },
		/// Event generated when the proof has been rejected
		ProofRejected { task_id: TaskId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The proof verification results list is full
		VerificationResultListIsFull,
		/// The verification of a proof has already been requested and is currently pending
		VerificationIsPending,
		/// The NeuroZK task for which the proof is requested doesn't exist
		NeuroZkTaskDoesNotExist,
		/// There are no authenticated verifiers
		NoAuthenticatedVerifiers,
		/// The authenticated verifier doesn't exist
		VerifierDoesNotExist,
		/// The verifiers verification task list is full
		VerificationTaskListIsFull,
		/// Too many finalizations scheduled per block
		TooManyScheduledFinalizations,
		/// The account trying to add verification results is not an authenticated verifier
		NotAnAuthenticatedVerifier,
		/// The account that should be added as a verifier is already an authenticated verifier
		AlreadyAnAuthenticatedVerifier
	}
	
	/// Used to aggregate verification results in a convenient mapping of `TaskId` to `VerificationStatusAggregator`,
	/// wich is three `BoundedVec`s, one for each verification status variant (Pending, Verified, Rejected), each
	/// containing a `BoundedVec` of the off-chain workers `AccountId`
	#[pallet::storage]
	pub type ConsensusAggregator<T: Config> = StorageMap<
		_,
		Twox64Concat,
		TaskId,
		VerificationStatusAggregator<T::AccountId>,
		OptionQuery,
	>;

	/// Used by the miner to submit proofs of execution when requested
	#[pallet::storage]
	pub type NeuroZkTaskDetails<T: Config> = StorageMap<_, Identity, TaskId, NeuroZkTaskInfo, OptionQuery>;

	/// Map of authenticated verifiers
	#[pallet::storage]
	pub type AuthenticatedVerifiers<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

	/// Queue containing all the task queues up for verification
	#[pallet::storage]
	pub type TaskQueue<T: Config> = StorageValue<_, BoundedVec<TaskId, T::MaxTasks>, ValueQuery>;

	/// The tasks that have been moved from `TaskQueue` to be processed in this block
	#[pallet::storage]
	#[pallet::getter(fn tasks_per_block)]
	pub type TasksPerBlock<T: Config> = StorageValue<_, BoundedVec<TaskId, T::MaxTasksPerBlock>, ValueQuery>;

	/// Map of tasks to be finalized at a certain block, giving verifiers time to submit their results
	#[pallet::storage]
	pub type FinalizationQueue<T: Config> = StorageMap<
		_,
		Twox64Concat,
		BlockNumberFor<T>,
		BoundedVec<TaskId, T::MaxScheduledFinalizations>,
		ValueQuery,
	>;

	/// Enables efficient checking of whether enough verifiers are present
	#[pallet::storage]
	pub type CurrentNumberOfVerifiers<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Lets the user request a proof from the miner which then gets verified by the offchain workers / verifying daemons
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn request_proof_verification(
			origin: OriginFor<T>,
			task_id: TaskId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::check_verification_requirements(task_id)?;
			
			NeuroZkTaskDetails::<T>::try_mutate_exists(task_id, |maybe_task_info| {
        		match maybe_task_info {
            		Some(task_info) => {
                		// Mutate the status
                		task_info.status = ProofVerificationStatus::Requested;
                		Ok::<(), DispatchError>(())
            		}
            		None => Err(Error::<T>::NeuroZkTaskDoesNotExist.into()),
        		}
    		});
				
			Self::deposit_event(Event::ProofRequested { task_id });

			//TODO: Once we have a proper consensus mechanism, the requests should be documented and the miners that don't deliver on them after a certain period slashed

			Ok(().into())
		}

		/// Lets the miner submit a proof and initiates the verification process
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn submit_proof(
			origin: OriginFor<T>,
			task_id: TaskId,
			proof: ZkProof,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			//TODO: Check if the tasks queue still has space left

			Self::check_verification_requirements(task_id)?;

			NeuroZkTaskDetails::<T>::try_mutate_exists(task_id, |maybe_task_info| {
        		match maybe_task_info {
            		Some(task_info) => {
                		// Mutate the status
                		task_info.status = ProofVerificationStatus::Pending;
						task_info.zk_proof = Some(proof);
                		Ok::<(), DispatchError>(())
            		}
            		None => Err(Error::<T>::NeuroZkTaskDoesNotExist.into()),
        		}
    		});

			TaskQueue::<T>::mutate(|tasks| {
				tasks.try_push(task_id).map_err(|_| Error::<T>::VerificationTaskListIsFull)
			})?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let finalize_at = current_block + T::FinalizationDelay::get();

			//TODO: The numbers that get put in here need to be thought out carefully
			FinalizationQueue::<T>::mutate(finalize_at, |tasks| {
				tasks.try_push(task_id).map_err(|_| Error::<T>::TooManyScheduledFinalizations)
			})?;

			//TODO: Find way to track verifiers that are active but don't submit verification results

			Ok(().into())
		}

		/// Submits list of `TaskId`s and their corresponding `VerificationStatus` from the Verifying Daemon, removes the processed tasks from the queue
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn submit_verification_results(
			origin: OriginFor<T>,
			results: VerifiedTasks<T::MaxTasksPerBlock>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				AuthenticatedVerifiers::<T>::contains_key(&who), 
				Error::<T>::NotAnAuthenticatedVerifier
			);
			
			Self::add_results(who, results)?;

			Ok(().into())
		}

		/// Lets collator nodes add a certain amount of BORG to let their ocws participate in verification
		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn stake_borg_and_start_verifying(
			origin: OriginFor<T>,
			offchain_worker_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!AuthenticatedVerifiers::<T>::contains_key(&offchain_worker_account), 
				Error::<T>::AlreadyAnAuthenticatedVerifier
			);

			AuthenticatedVerifiers::<T>::insert(&offchain_worker_account, ());
			CurrentNumberOfVerifiers::<T>::mutate(|n| *n += 1);

			//TODO Implement staking logic

			Ok(().into())
		}

		/// Lets collator nodes opt out of verification
		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn opt_out_of_verifying(
			origin: OriginFor<T>,
			offchain_worker_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
        		AuthenticatedVerifiers::<T>::contains_key(&offchain_worker_account),
        		Error::<T>::NotAnAuthenticatedVerifier
    		);

    		AuthenticatedVerifiers::<T>::remove(&offchain_worker_account);
			CurrentNumberOfVerifiers::<T>::mutate(|n| *n -= 1);

    		// TODO: Implement unstaking logic

    		Ok(().into())	
		}
		
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain Worker entry point.
		///
		/// This function will be called when the node is fully synced and a new best block is
		/// successfully imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			log::info!("Hello World from offchain workers!");

			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!(
				"Current block: {:?} (parent hash: {:?})",
				block_number,
				parent_hash
			);

			Self::fetch_verification_result_and_send_signed();
		}	

		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let tasks_to_finalize = FinalizationQueue::<T>::take(now);
	
			for task_id in tasks_to_finalize {
				if let Err(e) = Self::finalize_verification(task_id) {
					log::error!("Failed to finalize verification {}: {:?}", task_id, e);
				}
			}
	
			//TODO: The wieght returned here needs to be adjusted for production
			T::DbWeight::get().reads_writes(1, 1)
		}	
	}
}

impl<T: Config> Pallet<T> {
	/// A helper function to fetch the verification result from the Verifying Daemon
	fn fetch_verification_result_and_send_signed() -> Result<(), &'static str>
	where
		T::MaxTasksPerBlock: Get<u32>,
	{
		let signer = Signer::<T, T::AuthorityId>::all_accounts();

		if !signer.can_sign() {
				log::error!("No local accounts available. Consider adding one via `author_insertKey` RPC.");
		}

		// Note this call will block until response is received.
		let results =
			Self::fetch_verification_result().map_err(|_| "Failed to fetch verification result")?;

		let submission_result = signer.send_signed_transaction(|_account| {
			Call::submit_verification_results { results: results.clone() }
		});

		for (acc, res) in &submission_result {
			match res {
				Ok(()) => log::info!("[{:?}] Submitted result {:?}", acc.id, res),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	fn fetch_verification_result() -> Result<NodeProofResponse<T::MaxTasksPerBlock>, http::Error>
	where
		T::MaxTasksPerBlock: Get<u32>,	
	{
		let tasks = Self::tasks_per_block();

		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

		let encoded_tasks: Vec<u8> = tasks.encode();

		let request = http::Request::get("http://127.0.0.1:6666/verify");

		// Set deadline?
		let pending = request
			.deadline(deadline)
			.send()
			.map_err(|_| http::Error::IoError)?;

		// Set deadline? Note that this is different from the request deadline
		let response: http::Response = pending
			.try_wait(deadline)
			.map_err(|_| http::Error::DeadlineReached)?
			.map_err(|_| http::Error::DeadlineReached)?;

		if response.code as u16 != 200u16 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		let body = response.body().collect::<Vec<u8>>();

		let decoded: NodeProofResponse<T::MaxTasksPerBlock> = BoundedVec::decode(&mut &body[..])
			.map_err(|_| http::Error::Unknown)?;

		Ok(decoded)
	}

	/// Add new result to the list
	fn add_results(
		who: T::AccountId,
		results: NodeProofResponse<T::MaxTasksPerBlock>,
	) -> Result<(), Error<T>> {
		for (task_id, is_verified) in results.clone().into_iter() {
			<ConsensusAggregator<T>>::try_mutate(task_id, |maybe_aggregator| -> Result<(), Error<T>> {
				let who = who.clone();
		
				let aggregator = maybe_aggregator.get_or_insert_with(|| VerificationStatusAggregator {
					pending: BoundedVec::default(),
					verified: BoundedVec::default(),
					rejected: BoundedVec::default(),
				});
		
				let target_vec = if is_verified {
					&mut aggregator.verified
				} else {
					&mut aggregator.rejected
				};
		
				target_vec
					.try_push(who.clone())
					.map_err(|_| Error::<T>::VerificationResultListIsFull)?;
		
				Ok(())
			})?;
		}

		Self::deposit_event(Event::NewResult { results, who });

		Ok(())
	}

	fn finalize_verification(task_id: TaskId) -> Result<(), Error<T>> {
		let consensus_aggegator = ConsensusAggregator::<T>::get(task_id);
	
		//TODO Implement logic for reaching consensus

		//TODO Update queues

		//TODO SECONDARY implement slashing

		todo!();
	}

	fn check_verification_requirements(task_id: TaskId) -> Result<(), Error<T>> {
		let task = NeuroZkTaskDetails::<T>::get(task_id).ok_or(Error::<T>::NeuroZkTaskDoesNotExist)?;

		ensure!(
			CurrentNumberOfVerifiers::<T>::get() > 0,
			Error::<T>::NoAuthenticatedVerifiers
		);

		ensure!(
			task.status != ProofVerificationStatus::Pending,
			Error::<T>::VerificationIsPending
		);

		Ok(())
	}

	fn slash_reserved(who: &T::AccountId, amount: u64) {
		//TODO: Implement slashing
		todo!();
	}

	pub fn insert_neuro_zk_submission_details(task_id: TaskId, zk_verifier_files: NeuroZkTaskSubmissionDetails) {
		let task_data = NeuroZkTaskInfo {
			zk_input: zk_verifier_files.zk_input,
			zk_settings: zk_verifier_files.zk_settings,
			zk_verifying_key: zk_verifier_files.zk_verifying_key,
			zk_proof: None,
			status: ProofVerificationStatus::Init,
		};

		NeuroZkTaskDetails::<T>::insert(task_id ,task_data);
	}

	pub fn retrieve_verification_data(task_id: TaskId) -> Option<NeuroZkTaskInfo> {
		NeuroZkTaskDetails::<T>::get(task_id)
	}

	pub fn retrieve_current_tasks() -> BoundedVec<TaskId, T::MaxTasksPerBlock> {
		TasksPerBlock::<T>::get()
	}
}