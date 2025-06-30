use crate::{mock::*, Error};
use crate::{
	RequestedProofs, VerificationResultsPerProof, SubmittedPerProof
};

use frame_support::{assert_ok, assert_noop, pallet_prelude::ConstU32, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;

use cyborg_primitives::{
	task::*,
	zkml::*,
};

fn create_neurozk_task(task_id: TaskId) {
	let who: AccountId = 1;
	let deposit = 10;
	let data_str = "1";
	let data_vec = data_str.as_bytes().to_vec();
	let data: BoundedVec<u8, ConstU32<5000>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let vk: BoundedVec<u8, ConstU32<500000>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let metadata: BoundedVec<u8, ConstU32<500>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let nzk_data = Some(NzkData {
		zk_input: data.clone(),
		zk_settings: data.clone(),
		zk_verifying_key: vk,
		zk_proof: None,
		last_proof_accepted: None,
	});

	let task_info = TaskInfo {
		task_owner: who,
		create_block: 1,
		metadata,
		nzk_data,
		time_elapsed: None,
		average_cpu_percentage_use: None,
		task_kind: TaskKind::NeuroZK,
		result: None, 
		compute_hours_deposit: Some(deposit),
		consume_compute_hours: None,
		task_status: TaskStatusType::Assigned, 
	};

	pallet_task_management::Tasks::<Test>::insert(task_id, task_info)
}

fn create_non_neurozk_task(task_id: TaskId) {
	let who: AccountId = 1;
	let deposit = 10;
	let data_str = "1";
	let data_vec = data_str.as_bytes().to_vec();
	let data: BoundedVec<u8, ConstU32<5000>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let vk: BoundedVec<u8, ConstU32<500000>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let metadata: BoundedVec<u8, ConstU32<500>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let nzk_data = Some(NzkData {
		zk_input: data.clone(),
		zk_settings: data.clone(),
		zk_verifying_key: vk,
		zk_proof: None,
		last_proof_accepted: None,
	});

	let task_info = TaskInfo {
		task_owner: who,
		create_block: 1,
		metadata,
		nzk_data,
		time_elapsed: None,
		average_cpu_percentage_use: None,
		task_kind: TaskKind::OpenInference,
		result: None, 
		compute_hours_deposit: Some(deposit),
		consume_compute_hours: None,
		task_status: TaskStatusType::Assigned, 
	};

	pallet_task_management::Tasks::<Test>::insert(task_id, task_info)
}

fn get_nzk_task(task_id: TaskId) -> Option<TaskInfo<AccountId, BlockNumberFor<Test>>> {
	pallet_task_management::Tasks::<Test>::get(task_id)
}

#[test]
fn request_proof_works_for_valid_task() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);

		assert_ok!(NeuroZk::request_proof(RuntimeOrigin::signed(1), task_id));
		assert_eq!(RequestedProofs::<Test>::get(task_id), Some(ProofVerificationStage::Requested));
	});
}

#[test]
fn request_proof_fails_if_already_requested() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);

		assert_ok!(NeuroZk::request_proof(RuntimeOrigin::signed(1), task_id));
		assert_noop!(
			NeuroZk::request_proof(RuntimeOrigin::signed(1), task_id),
			Error::<Test>::ProofAlreadyRequested
		);
	});
}

#[test]
fn request_proof_fails_for_invalid_task_type() {
	new_test_ext().execute_with(|| {
		let task_id = 2;
		create_non_neurozk_task(task_id);

		assert_noop!(
			NeuroZk::request_proof(RuntimeOrigin::signed(1), task_id),
			Error::<Test>::InvalidTaskType
		);
	});
}

#[test]
fn request_proof_fails_for_nonexistent_task() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			NeuroZk::request_proof(RuntimeOrigin::signed(1), 99),
			Error::<Test>::TaskDoesNotExist
		);
	});
}

#[test]
fn submit_proof_works() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);
		let proof = Default::default();

		assert_ok!(NeuroZk::request_proof(RuntimeOrigin::signed(1), task_id));
		assert_ok!(NeuroZk::submit_proof(RuntimeOrigin::signed(2), task_id, proof));

		assert_eq!(RequestedProofs::<Test>::get(task_id), Some(ProofVerificationStage::Pending));
	});
}

#[test]
fn submit_proof_fails_if_not_requested() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);

		let proof = Default::default();
		assert_noop!(
			NeuroZk::submit_proof(RuntimeOrigin::signed(1), task_id, proof),
			Error::<Test>::ProofNotRequested
		);
	});
}

#[test]
fn submit_proof_fails_if_already_submitted() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);
		let proof_str = "1";
		let proof_vec = proof_str.as_bytes().to_vec();
		let proof: BoundedVec<u8, ConstU32<50000>> = BoundedVec::try_from(proof_vec.clone()).unwrap();

		assert_ok!(NeuroZk::request_proof(RuntimeOrigin::signed(1), task_id));
		assert_ok!(NeuroZk::submit_proof(RuntimeOrigin::signed(2), task_id, proof.clone()));
		
		assert_noop!(
			NeuroZk::submit_proof(RuntimeOrigin::signed(2), task_id, proof),
			Error::<Test>::ProofAlreadySubmitted
		);
	});
}

#[test]
fn submit_proof_fails_for_invalid_task_type() {
	new_test_ext().execute_with(|| {
		let task_id = 2;
		create_non_neurozk_task(task_id);

		let proof_str = "1";
		let proof_vec = proof_str.as_bytes().to_vec();
		let proof: BoundedVec<u8, ConstU32<50000>> = BoundedVec::try_from(proof_vec.clone()).unwrap();

		assert_noop!(
			NeuroZk::submit_proof(RuntimeOrigin::signed(2), task_id, proof),
			Error::<Test>::ProofNotRequested
		);
	});
}

#[test]
fn on_new_data_records_and_finalizes_correctly() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);
		let feeder1 = 10;
		let feeder2 = 11;

		// Push values until we reach threshold
		NeuroZk::on_new_data(&feeder1, &task_id, &true);
		NeuroZk::on_new_data(&feeder2, &task_id, &true);

		let stored_results = VerificationResultsPerProof::<Test>::get(task_id);
		assert_eq!(stored_results.len(), 2);

		assert_eq!(
			SubmittedPerProof::<Test>::get((feeder1, task_id)),
			true
		);
	});
}

#[test]
fn on_new_data_ignores_duplicate_submitters() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);
		let feeder = 10;

		NeuroZk::on_new_data(&feeder, &task_id, &true);
		NeuroZk::on_new_data(&feeder, &task_id, &true); // should be ignored

		let stored_results = VerificationResultsPerProof::<Test>::get(task_id);
		assert_eq!(stored_results.len(), 1);
	});
}

#[test]
fn finalize_verification_accepts_on_threshold() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);

		let feeders = vec![10, 11, 12, 13, 14];
		for acc in &feeders {
			NeuroZk::on_new_data(acc, &task_id, &true);
		}

		let task = get_nzk_task(task_id).unwrap();
		assert_eq!(task.nzk_data.unwrap().last_proof_accepted.unwrap().0, true);
	});
}

#[test]
fn finalize_verification_rejects_below_threshold() {
	new_test_ext().execute_with(|| {
		let task_id = 1;
		create_neurozk_task(task_id);

		let votes = vec![(10, true), (11, false), (12, false), (13, false), (14, false)];
		for (acc, vote) in votes {
			NeuroZk::on_new_data(&acc, &task_id, &vote);
		}

		let task = get_nzk_task(task_id).unwrap();
		assert_eq!(task.nzk_data.unwrap().last_proof_accepted.unwrap().0, false);
	});
}