#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::vec;
use sp_std::prelude::*;
use cyborg_primitives::task::*;
use frame_support::pallet_prelude::ConstU32;
use frame_system::pallet_prelude::BlockNumberFor;

fn create_neurozk_task<T: Config + pallet_task_management::Config>(task_id: TaskId) {
	let who: T::AccountId = account("user", 0, 0);
	let deposit = 10;
	let data_str = "1";
	let data_vec = data_str.as_bytes().to_vec();
	let data: BoundedVec<u8, ConstU32<5000>> = BoundedVec::try_from(data_vec.clone()).unwrap();
    let vk: BoundedVec<u8, ConstU32<500000>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let metadata: BoundedVec<u8, ConstU32<500>> = BoundedVec::try_from(data_vec.clone()).unwrap();
	let nzk_data = Some(NzkData {
		zk_input: data.clone(),
		zk_settings: data,
		zk_verifying_key: vk,
		zk_proof: None,
		last_proof_accepted: None,
	});

	let taskInfo = TaskInfo::<T::AccountId, BlockNumberFor<T>> {
		task_owner: who,
		create_block: <frame_system::Pallet<T>>::block_number(),
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

	pallet_task_management::Tasks::<T>::insert(task_id, taskInfo)
}

#[benchmarks(
	where
		T: Config + pallet_task_management::Config
)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn request_proof<T: Config + pallet_task_management::Config>() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();
        let task_id = 0;

        create_neurozk_task::<T>(task_id.clone());

        #[block]
        {
            Pallet::<T>::request_proof(RawOrigin::Signed(caller.clone()).into(), task_id.clone())?;
        }

        assert_eq!(RequestedProofs::<T>::get(&task_id), Some(ProofVerificationStage::Requested));
        Ok(())
    }

    #[benchmark]
    fn submit_proof<T: Config + pallet_task_management::Config>() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();
        let task_id = 0;
        let dummy_bytes = vec![1u8; 50_000]; // 50KB each
		let dummy_proof = BoundedVec::try_from(dummy_bytes.clone()).unwrap();


        create_neurozk_task::<T>(task_id.clone());
        RequestedProofs::<T>::insert(&task_id, ProofVerificationStage::Requested);

        #[block]
        {
            Pallet::<T>::submit_proof(RawOrigin::Signed(caller.clone()).into(), task_id.clone(), dummy_proof.clone())?;
        }

        assert_eq!(RequestedProofs::<T>::get(&task_id), Some(ProofVerificationStage::Pending));
        Ok(())
    }

    #[benchmark]
    fn on_new_data_finalize<T: Config + pallet_task_management::Config>() -> Result<(), BenchmarkError> {
        let who: T::AccountId = whitelisted_caller();
        let task_id = 0;

		create_neurozk_task::<T>(task_id.clone());
        RequestedProofs::<T>::insert(&task_id, ProofVerificationStage::Requested);

        #[block]
        {
            for i in 0..T::AggregateLength::get() {
                let oracle = T::AccountId::decode(&mut &i.to_le_bytes()[..]).unwrap_or_else(|_| who.clone());
                Pallet::<T>::on_new_data(&oracle, &task_id, &true);
            }
        }

        // Finalize step should complete and result should be stored
        let results = VerificationResultsPerProof::<T>::get(&task_id);
        assert_eq!(results.len() as u32, T::AggregateLength::get());
        Ok(())
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}