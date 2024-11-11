// MIT License

// Copyright (c) 2022 Bright Inventions

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// MODIFICATIONS:
// - Modified by ZCalz on 2024-10-20: Updated storage structures

#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[macro_use]
extern crate uint;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;
pub use weights::*;

pub mod common;
pub mod deserialization;
pub mod verify;

use frame_support::storage::bounded_vec::BoundedVec;
use sp_std::vec::Vec;

type TaskId = u64;
type PublicInputsDef<T> = BoundedVec<u8, <T as Config>::MaxPublicInputsLength>;
type ProofDef<T> = BoundedVec<u8, <T as Config>::MaxProofLength>;
type VerificationKeyDef<T> = BoundedVec<u8, <T as Config>::MaxVerificationKeyLength>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::{
		common::prepare_verification_key,
		deserialization::{deserialize_public_inputs, Proof, VKey},
		verify::{
			prepare_public_inputs, verify, G1UncompressedBytes, G2UncompressedBytes, GProof,
			VerificationKey, SUPPORTED_CURVE, SUPPORTED_PROTOCOL,
		},
	};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	// #[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxPublicInputsLength: Get<u32>;

		/// The maximum length of the proof.
		#[pallet::constant]
		type MaxProofLength: Get<u32>;

		/// The maximum length of the verification key.
		#[pallet::constant]
		type MaxVerificationKeyLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VerificationSetupCompleted,
		VerificationProofSet,
		VerificationSuccess { who: T::AccountId },
		VerificationFailed,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Public inputs mismatch
		PublicInputsMismatch,
		/// Public inputs vector is to long.
		TooLongPublicInputs,
		/// The verification key is to long.
		TooLongVerificationKey,
		/// The proof is too long.
		TooLongProof,
		/// The proof is too short.
		ProofIsEmpty,
		/// Verification key, not set.
		VerificationKeyIsNotSet,
		/// Malformed key
		MalformedVerificationKey,
		/// Malformed proof
		MalformedProof,
		/// Malformed public inputs
		MalformedPublicInputs,
		/// Curve is not supported
		NotSupportedCurve,
		/// Protocol is not supported
		NotSupportedProtocol,
		/// There was error during proof verification
		ProofVerificationError,
		/// Proof creation error
		ProofCreationError,
		/// Verification Key creation error
		VerificationKeyCreationError,
	}

	/// Storing a public input.
	#[pallet::storage]
	#[pallet::getter(fn public_input)]
	pub type PublicInputStorage<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, PublicInputsDef<T>, OptionQuery>;

	/// Storing a proof.
	#[pallet::storage]
	#[pallet::getter(fn proof)]
	pub type ProofStorage<T: Config> = StorageMap<_, Twox64Concat, TaskId, ProofDef<T>, OptionQuery>;

	/// Storing a verification key.
	#[pallet::storage]
	#[pallet::getter(fn verification_key)]
	pub type VerificationKeyStorage<T: Config> =
		StorageMap<_, Twox64Concat, TaskId, VerificationKeyDef<T>, OptionQuery>;
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Store a verification key.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::setup_verification_benchmark())]
		pub fn setup_verification(
			_origin: OriginFor<T>,
			task_id: TaskId,
			pub_input: Vec<u8>,
			vec_vk: Vec<u8>,
		) -> DispatchResult {
			// Store the public inputs and verification key under the given TaskId.
			let inputs = store_public_inputs::<T>(task_id, pub_input)?;
			let vk = store_verification_key::<T>(task_id, vec_vk)?;

			// Ensure the length of the inputs matches the expected length from the verification key.
			ensure!(
				vk.public_inputs_len == inputs.len() as u8,
				Error::<T>::PublicInputsMismatch
			);

			// Emit an event to signal that verification setup is complete.
			Self::deposit_event(Event::<T>::VerificationSetupCompleted);

			Ok(())
		}

		/// Verify a proof.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::verify_benchmark())]
		pub fn verify(origin: OriginFor<T>, task_id: TaskId, vec_proof: Vec<u8>) -> DispatchResult {
			let proof = store_proof::<T>(task_id, vec_proof)?;
			let vk = get_verification_key::<T>(task_id)?;
			let inputs = get_public_inputs::<T>(task_id)?;
			let sender = ensure_signed(origin)?;
			Self::deposit_event(Event::<T>::VerificationProofSet);

			match verify(vk, proof, prepare_public_inputs(inputs)) {
				Ok(true) => {
					Self::deposit_event(Event::<T>::VerificationSuccess { who: sender });
					Ok(())
				}
				Ok(false) => {
					Self::deposit_event(Event::<T>::VerificationFailed);
					Ok(())
				}
				Err(_) => Err(Error::<T>::ProofVerificationError.into()),
			}
		}
	}

	fn get_public_inputs<T: Config>(task_id: TaskId) -> Result<Vec<u64>, sp_runtime::DispatchError> {
		let public_inputs =
			PublicInputStorage::<T>::get(task_id).ok_or(Error::<T>::PublicInputsMismatch)?;
		let deserialized_public_inputs = deserialize_public_inputs(public_inputs.as_slice())
			.map_err(|_| Error::<T>::MalformedPublicInputs)?;
		Ok(deserialized_public_inputs)
	}

	fn store_public_inputs<T: Config>(
		task_id: TaskId,
		pub_input: Vec<u8>,
	) -> Result<Vec<u64>, sp_runtime::DispatchError> {
		let public_inputs: PublicInputsDef<T> = pub_input
			.try_into()
			.map_err(|_| Error::<T>::TooLongPublicInputs)?;
		let deserialized_public_inputs = deserialize_public_inputs(public_inputs.as_slice())
			.map_err(|_| Error::<T>::MalformedPublicInputs)?;
		PublicInputStorage::<T>::insert(task_id, public_inputs);
		Ok(deserialized_public_inputs)
	}

	fn get_verification_key<T: Config>(
		task_id: TaskId,
	) -> Result<VerificationKey, sp_runtime::DispatchError> {
		let vk =
			VerificationKeyStorage::<T>::get(task_id).ok_or(Error::<T>::VerificationKeyIsNotSet)?;
		let deserialized_vk =
			VKey::from_json_u8_slice(vk.as_slice()).map_err(|_| Error::<T>::MalformedVerificationKey)?;
		let vk = prepare_verification_key(deserialized_vk)
			.map_err(|_| Error::<T>::VerificationKeyCreationError)?;
		Ok(vk)
	}

	fn store_verification_key<T: Config>(
		task_id: TaskId,
		vec_vk: Vec<u8>,
	) -> Result<VKey, sp_runtime::DispatchError> {
		let vk: VerificationKeyDef<T> = vec_vk
			.try_into()
			.map_err(|_| Error::<T>::TooLongVerificationKey)?;
		let deserialized_vk =
			VKey::from_json_u8_slice(vk.as_slice()).map_err(|_| Error::<T>::MalformedVerificationKey)?;
		ensure!(
			deserialized_vk.curve == SUPPORTED_CURVE.as_bytes(),
			Error::<T>::NotSupportedCurve
		);
		ensure!(
			deserialized_vk.protocol == SUPPORTED_PROTOCOL.as_bytes(),
			Error::<T>::NotSupportedProtocol
		);
		VerificationKeyStorage::<T>::insert(task_id, vk);
		Ok(deserialized_vk)
	}

	fn store_proof<T: Config>(
		task_id: TaskId,
		vec_proof: Vec<u8>,
	) -> Result<GProof, sp_runtime::DispatchError> {
		// Ensure proof is not empty.
		ensure!(!vec_proof.is_empty(), Error::<T>::ProofIsEmpty);

		// Try to convert the vector into the expected proof definition.
		let proof: ProofDef<T> = vec_proof.try_into().map_err(|_| Error::<T>::TooLongProof)?;

		// Deserialize the proof from JSON-encoded bytes.
		let deserialized_proof =
			Proof::from_json_u8_slice(proof.as_slice()).map_err(|_| Error::<T>::MalformedProof)?;

		// Ensure the curve used is supported.
		ensure!(
			deserialized_proof.curve == SUPPORTED_CURVE.as_bytes(),
			Error::<T>::NotSupportedCurve
		);

		// Ensure the protocol used is supported.
		ensure!(
			deserialized_proof.protocol == SUPPORTED_PROTOCOL.as_bytes(),
			Error::<T>::NotSupportedProtocol
		);

		// Store the proof in storage associated with the task_id.
		ProofStorage::<T>::insert(task_id, proof.clone());

		// Construct the GProof from the deserialized data.
		let g_proof = GProof::from_uncompressed(
			&G1UncompressedBytes::new(deserialized_proof.a[0], deserialized_proof.a[1]),
			&G2UncompressedBytes::new(
				deserialized_proof.b[0][0],
				deserialized_proof.b[0][1],
				deserialized_proof.b[1][0],
				deserialized_proof.b[1][1],
			),
			&G1UncompressedBytes::new(deserialized_proof.c[0], deserialized_proof.c[1]),
		)
		.map_err(|_| Error::<T>::ProofCreationError)?;

		// Return the constructed proof.
		Ok(g_proof)
	}
}
