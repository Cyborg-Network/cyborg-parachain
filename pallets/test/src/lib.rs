#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::traits::Get;
use frame_system::{
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction,
		SendUnsignedTransaction, SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::offchain::{
		http,
		Duration,
};

pub use cyborg_primitives::{
    proof::*, 
    task::{TaskId, TaskType}
};
use pallet_task_management::Tasks;

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
	app_crypto!(sr25519, KEY_TYPE);

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

  #[pallet::error]
  pub enum Error<T> {
    /// The proof verification results list is full
    VerificationResultListIsFull,
    /// The verification of a proof has already been requested and is currently pending
    VerificationIsPending,
    /// The NeuroZK task for which the proof is requested doesn't exist
    NeuroZkTaskDoesNotExist
  }

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_task_management::Config
	{
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// Configuration parameters

		/// A grace period after we send transaction.
		///
		/// To avoid sending too many transactions, we only attempt to send one
		/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
		/// sending between distinct runs of this offchain worker.
		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;

    	/// Limit for the `BoundedVec` of how many authenticated verifiers there can be
    	#[pallet::constant]
    	type MaxNumOfAuthenticatedVerifiers: Get<u32>;

    	/// Max number of proof verifiers, this is how many accounts will be assigned for verification
		#[pallet::constant]
		type MaxNumOfProofVerifiers: Get<u32>;

    	/// Of the max `MaxNumOfVerifiers` there might be verifiers that are offline, so if the
    	/// `MinNumberOfVerifiers` isn't reached, the verification process will be repeated
    	#[pallet::constant]
		type MinNumOfProofVerifiers: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain Worker entry point.
		///
		/// By implementing `fn offchain_worker` you declare a new offchain worker.
		/// This function will be called when the node is fully synced and a new best block is
		/// successfully imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			// Note that having logs compiled to WASM may cause the size of the blob to increase
			// significantly. You can use `RuntimeDebug` custom derive to hide details of the types
			// in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
			// all logging and thus, remove any logging from the WASM.
			log::info!("Hello World from offchain workers!");

			// Since off-chain workers are just part of the runtime code, they have direct access
			// to the storage and other included pallets.
			//
			// We can easily import `frame_system` and retrieve a block hash of the parent block.
			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			// Call helper function to query the node to verify the proof
		  	fetch_verification_result_and_send_signed();
		}
	}

	/// A public part of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit new price to the list.
		///
		/// This method is a public function of the module and can be called from within
		/// a transaction. It appends given `price` to current list of prices.
		/// In our example the `offchain worker` will create, sign & submit a transaction that
		/// calls this function passing the price.
		///
		/// The transaction needs to be signed (see `ensure_signed`) check, so that the caller
		/// pays a fee to execute it.
		/// This makes sure that it's not easy (or rather cheap) to attack the chain by submitting
		/// excessive transactions, but note that it doesn't ensure the price oracle is actually
		/// working and receives (and provides) meaningful data.
		/// This example is not focused on correctness of the oracle itself, but rather its
		/// purpose is to showcase offchain worker capabilities.
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn submit_result(origin: OriginFor<T>, result: VerificationResult) -> DispatchResultWithPostInfo {
			// Retrieve sender of the transaction.
			let who = ensure_signed(origin)?;
			// Add the price to the on-chain list.
			Self::add_result(who, result)?;
			Ok(().into())
		}

    #[pallet::call_index(1)]
		#[pallet::weight({0})]
    pub fn request_proof_verification(origin: OriginFor<T>, task_id: TaskId, proof: Proof) -> DispatchResultWithPostInfo {
      let who = ensure_signed(origin)?;

      let task = pallet_task_management::Pallet::<T>::Tasks::get(task_id)
        .ok_or(Error::<T>::NeuroZkTaskDoesNotExist)?;
      
      ensure!(
          matches!(task.task_type, TaskType::ZK), 
          Error::<T>::NeuroZkTaskDoesNotExist
      );

      if let Some((proof, status)) = Proofs::<T>::get(task_id) {
        ensure!(status != ProofVerificationStatus::Pending, Error::<T>::VerificationIsPending);
      } else {
        //TODO: Randomly assign max proof verifiers list from `AuthenticatedVerifiers`
        todo!();
        //return Error::<T>::
      };
      
      Ok(())
    }
  }

	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when new price is accepted to contribute to the average.
		NewResult { result: VerificationResult, who: T::AccountId },
    /// event generated when the proof has successfully been verified
    ProofVerified { task_id: TaskId },
    /// Event generated when the proof has been rejected
    ProofRejected { task_id: TaskId },
	}

	/// This is used to aggregate verification results and make a decision.
	#[pallet::storage]
	pub type VerificationAggregator<T: Config> = StorageMap<_, Twox64Concat, TaskId, BoundedVec<(T::AccountId, VerificationResult), T::MaxNumOfProofVerifiers>, OptionQuery>;

  /// This is used by the miner to submit proofs of execution, and by ocw to update verification status
  #[pallet::storage]
  pub type Proofs<T: Config> = StorageMap<_, Identity, TaskId, (Proof, ProofVerificationStatus), OptionQuery>;

  /// This is a list of all accounts that are allowed for ocw proof verification
  #[pallet::storage]
  pub type AuthenticatedVerifiers<T: Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxNumOfAuthenticatedVerifiers>, ValueQuery>;
}

impl<T: Config> Pallet<T> {
	/// A helper function to fetch the price and send signed transaction.
	fn fetch_verification_result_and_send_signed() -> Result<(), &'static str> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC.",
			)
		}
		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let result = Self::fetch_verification_result().map_err(|_| "Failed to fetch price")?;

		// Using `send_signed_transaction` associated type we create and submit a transaction
		// representing the call, we've just created.
		// Submit signed will return a vector of results for all accounts that were found in the
		// local keystore with expected `KEY_TYPE`.
		let results = signer.send_signed_transaction(|_account| {
			// Received price is wrapped into a call to `submit_price` public function of this
			// pallet. This means that the transaction, when executed, will simply call that
			// function passing `price` as an argument.
			Call::submit_result { result }
		});

		for (acc, res) in &results {
			match res {
				Ok(()) => log::info!("[{:?}] Submitted result {}", acc.id, result),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	fn fetch_verification_result() -> Result<VerificationResult, http::Error> {
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

		let request =
			http::Request::get("http://127.0.0.1:6666/verify");

    // Set deadline?
		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

    // Set deadline? Not that this is different from the request deadline
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;

		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown)
		}

		let body = response.body().collect::<Vec<u8>>();

		let body_str = alloc::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;

    let value = match body_str.trim() {
	      "1" => Ok(true),
	      "0" => Ok(false),
	      _ => {
		        log::warn!("Unexpected boolean value: {:?}", body_str);
		        Err(http::Error::Unknown)
	      },
    }?;

		log::warn!("Proof correct: {}", value);

		Ok(value)
	}

	/// Add new result to the list
	fn add_result(who: T::AccountId, result: VerificationResult) -> Result<(), Error<T>>  {
		log::info!("Adding '{}' to results", result);
		<VerificationAggregator<T>>::mutate(|results| {
			if results.try_push(result).is_err() {
        return Err(Error::<T>::VerificationResultListIsFull);
			}
      Ok(())
		})?;

		Self::deposit_event(Event::NewResult { result, who });

    Ok(())
	}
}
