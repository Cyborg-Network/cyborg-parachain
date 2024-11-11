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

use crate::{
	deserialization::{Proof, VKey},
	verify::{
		G1UncompressedBytes, G2UncompressedBytes, GProof, GProofCreationError, VerificationKey,
		VerificationKeyCreationError,
	},
};
use sp_std::vec::Vec;

pub fn prepare_verification_key(
	deserialized_vk: VKey,
) -> Result<VerificationKey, VerificationKeyCreationError> {
	let mut ic: Vec<G1UncompressedBytes> = Vec::with_capacity(deserialized_vk.ic.len());
	for i in 0..deserialized_vk.ic.len() {
		let g1_bytes = G1UncompressedBytes::new(deserialized_vk.ic[i][0], deserialized_vk.ic[i][1]);
		ic.push(g1_bytes)
	}
	VerificationKey::from_uncompressed(
		&G1UncompressedBytes::new(deserialized_vk.alpha[0], deserialized_vk.alpha[1]),
		&G2UncompressedBytes::new(
			deserialized_vk.beta[0][0],
			deserialized_vk.beta[0][1],
			deserialized_vk.beta[1][0],
			deserialized_vk.beta[1][1],
		),
		&G2UncompressedBytes::new(
			deserialized_vk.gamma[0][0],
			deserialized_vk.gamma[0][1],
			deserialized_vk.gamma[1][0],
			deserialized_vk.gamma[1][1],
		),
		&G2UncompressedBytes::new(
			deserialized_vk.delta[0][0],
			deserialized_vk.delta[0][1],
			deserialized_vk.delta[1][0],
			deserialized_vk.delta[1][1],
		),
		&ic,
	)
}

pub fn prepare_proof(proof: Proof) -> Result<GProof, GProofCreationError> {
	GProof::from_uncompressed(
		&G1UncompressedBytes::new(proof.a[0], proof.a[1]),
		&G2UncompressedBytes::new(proof.b[0][0], proof.b[0][1], proof.b[1][0], proof.b[1][1]),
		&G1UncompressedBytes::new(proof.c[0], proof.c[1]),
	)
}
