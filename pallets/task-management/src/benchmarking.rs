#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;

use cyborg_primitives::worker::*;
use frame_support::{sp_runtime::traits::ConstU32, BoundedVec};
use frame_system::RawOrigin;
use scale_info::prelude::vec;

// Define a constant for the worker API domain.
const WORKER_API_DOMAIN1: &str = "https://api-worker1.testing";
const WORKER_API_DOMAIN2: &str = "https://api-worker2.testing";
const WORKER_API_DOMAIN3: &str = "https://api-worker3.testing";
const DOCKER_IMAGE_TESTDATA: &str = "some-docker-imgv.0";

// Helper function to convert the domain string into a BoundedVec with a maximum length of 128 bytes.
// Convert the domain string into a vector of bytes and then into a BoundedVec with a maximum length of 128 bytes.
// The `try_from` mthod ensure that the length of the string doesn't exceed the limit.
fn get_domain(domain_str: &str) -> BoundedVec<u8, ConstU32<128>> {
	BoundedVec::try_from(domain_str.as_bytes().to_vec())
		.expect("Domain string exceeds maximum length")
}

// Helper function to convert task data into a BoundedVec with a maximum length of 128 bytes.
// This ensures the task data string does not exceed the limit of 128 bytes.
fn get_taskdata(task_data_str: &str) -> BoundedVec<u8, ConstU32<500>> {
	BoundedVec::try_from(task_data_str.as_bytes().to_vec())
		.expect("Task Data string exceeds maximum length")
}

/// A function to initialize benchmarking data by creating multiple worker accounts with various attributes
/// for each worker, including location, specifications, and status.
///
/// The function generates 100 creators, and for each creator, it creates up to `MAX_WORKER_ID` workers.
/// It inserts these workers into the `WorkerClusters` map, associating them with their respective creators.
///
/// # Type Parameters
/// - `T`: The configuration trait for the pallet, which provides the necessary types like `AccountId`.
fn set_initial_benchmark_data<T: Config>() {
	for i in 0..100 {
		let index = i;
		let seed = i * 3;
		// Generate the creator account for this worker
		let creator = account::<T::AccountId>("benchmark_account", index, seed);

		// Create the WorkerAPI object
		let api = WorkerAPI {
			domain: match i % 3 {
				1 => get_domain(WORKER_API_DOMAIN1),
				2 => get_domain(WORKER_API_DOMAIN2),
				0 => get_domain(WORKER_API_DOMAIN3),
				_ => get_domain(WORKER_API_DOMAIN1),
			},
		};

		// Create the location object for the worker
		let worker_location = Location {
			latitude: i as i32,
			longitude: (103 + i) as i32,
		};

		// Define the worker's specifications (RAM, storage, CPU)
		let worker_specs = WorkerSpecs {
			ram: 5_000_000_000u64,
			storage: 100_000_000_000u64,
			cpu: 5u16,
		};

		// Maximum number of worker IDs to generate
		const MAX_WORKER_ID: u64 = 6;

		// Insert the MAX_WORKER_ID into the AccountWorkers map for the creator
		pallet_edge_connect::AccountWorkers::<T>::insert(creator.clone(), MAX_WORKER_ID);

		let worker_reputation = WorkerReputation {
			score: 0,
			last_updated: None,
			violations: 0,
			successful_tasks: 0,
		};

		// Loop to create multiple workers for the same creator
		for worker_id in 0..MAX_WORKER_ID {
			// Get the current block number
			let blocknumber = <frame_system::Pallet<T>>::block_number();

			// Create a Worker struct with all the required data
			let worker = Worker {
				id: worker_id,
				owner: creator.clone(),
				location: worker_location.clone(),
				specs: worker_specs.clone(),
				reputation: worker_reputation,
				start_block: blocknumber,
				status: WorkerStatusType::Inactive,
				status_last_updated: blocknumber,
				api: api.clone(),
				last_status_check: pallet_timestamp::Pallet::<T>::get(),
			};

			// Insert the worker into the WorkerClusters map
			pallet_edge_connect::WorkerClusters::<T>::insert((creator.clone(), worker_id), worker);

			// Initialize Compute Hours for the creator account in the payment pallet.
			pallet_payment::ComputeHours::<T>::insert(creator.clone(), 50);
		}
	}
}

#[benchmarks]
mod benchmarks {
	use super::*;
	use scale_info::prelude::vec::Vec;

	#[benchmark]
	fn task_scheduler_no_nzk<T: Config>() -> Result<(), BenchmarkError> {
		// Setup initial benchmark data (this function is assumed to prepopulate necessary state)
		// This function could be used to set up the initial state required for benchmarking.
		set_initial_benchmark_data::<T>();

		// Assign a caller account that will act as the worker's owner.
		let caller: T::AccountId = whitelisted_caller();
		// Create task data.
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);

		let worker_account = account::<T::AccountId>("benchmark_account", 0, 0);

		let worker_id = 0;

		// Initialize Compute Hours for the caller account in the payment pallet.
		// This ensures the account has sufficient compute hours for task operations during benchmarking.
		pallet_payment::ComputeHours::<T>::insert(caller.clone(), 50);

		#[block]
		{
			Pallet::<T>::task_scheduler(
				RawOrigin::Signed(caller.clone()).into(),
				TaskKind::OpenInference,
				task_data,
				None,
				worker_account,
				worker_id,
				Some(10),
			)
			.expect("Failed to schedule task");
		}

		// Verification code
		// We will check the worker's details by looking them up in the worker cluster storage map.
		let task_allocation = TaskAllocations::<T>::iter().collect::<Vec<_>>();
		assert!(task_allocation.len() > 0, "Failed to schedule task");

		let task_owner = TaskOwners::<T>::iter().collect::<Vec<_>>();
		assert!(task_owner.len() > 0, "Failed to schedule task");

		let tasks = Tasks::<T>::iter().collect::<Vec<_>>();
		assert!(tasks.len() > 0, "Failed to schedule task");

		let task_status = TaskStatus::<T>::iter().collect::<Vec<_>>();
		assert!(task_status.len() > 0, "Failed to schedule task");

		Ok(())
	}

	#[benchmark]
	fn task_scheduler_nzk<T: Config>() -> Result<(), BenchmarkError> {
		// Setup initial benchmark data (this function is assumed to prepopulate necessary state)
		// This function could be used to set up the initial state required for benchmarking.
		set_initial_benchmark_data::<T>();

		// Assign a caller account that will act as the worker's owner.
		let caller: T::AccountId = whitelisted_caller();
		// Create task data.
		let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);

		let worker_account = account::<T::AccountId>("benchmark_account", 0, 0);

		let worker_id = 0;

		// Initialize Compute Hours for the caller account in the payment pallet.
		// This ensures the account has sufficient compute hours for task operations during benchmarking.
		pallet_payment::ComputeHours::<T>::insert(caller.clone(), 50);

		let dummy_bytes = vec![1u8; 1_000_000]; // 1MB each
		let nzk_info = Some(NeuroZkTaskSubmissionDetails {
			zk_input: BoundedVec::try_from(dummy_bytes.clone()).unwrap(),
			zk_settings: BoundedVec::try_from(dummy_bytes.clone()).unwrap(),
			zk_verifying_key: BoundedVec::try_from(dummy_bytes).unwrap(),
		});

		#[block]
		{
			Pallet::<T>::task_scheduler(
				RawOrigin::Signed(caller.clone()).into(),
				TaskKind::NeuroZK,
				task_data,
				nzk_info,
				worker_account,
				worker_id,
				Some(10),
			)
			.expect("Failed to schedule task");
		}

		// Verification code
		// We will check the worker's details by looking them up in the worker cluster storage map.
		let task_allocation = TaskAllocations::<T>::iter().collect::<Vec<_>>();
		assert!(task_allocation.len() > 0, "Failed to schedule task");

		let task_owner = TaskOwners::<T>::iter().collect::<Vec<_>>();
		assert!(task_owner.len() > 0, "Failed to schedule task");

		let tasks = Tasks::<T>::iter().collect::<Vec<_>>();
		assert!(tasks.len() > 0, "Failed to schedule task");

		let task_status = TaskStatus::<T>::iter().collect::<Vec<_>>();
		assert!(task_status.len() > 0, "Failed to schedule task");

		Ok(())
	}

  #[benchmark]
    fn stop_task_and_vacate_miner<T: Config>() -> Result<(), BenchmarkError> {
		set_initial_benchmark_data::<T>();
		let caller: T::AccountId = whitelisted_caller();

        pallet_payment::ComputeHours::<T>::insert(caller.clone(), 100);
        let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);
        Pallet::<T>::task_scheduler(
            RawOrigin::Signed(caller.clone()).into(),
            TaskKind::OpenInference,
            task_data.clone(),
            None,
            caller.clone(),
            1,
            Some(5),
        )?;
        let task_id = NextTaskId::<T>::get() - 1;

        Pallet::<T>::confirm_task_reception(
            RawOrigin::Signed(caller.clone()).into(),
            task_id,
        )?;

        #[block]
        {
            Pallet::<T>::stop_task_and_vacate_miner(
                RawOrigin::Signed(caller.clone()).into(),
                task_id,
            )?;
        }

        assert_eq!(TaskStatus::<T>::get(task_id), Some(TaskStatusType::Stopped));
        assert!(ComputeAggregations::<T>::get(task_id).and_then(|(_, e)| e).is_some());
        Ok(())
    }

    #[benchmark]
    fn confirm_miner_vacation<T: Config>() -> Result<(), BenchmarkError> {
		set_initial_benchmark_data::<T>();
		let caller: T::AccountId = whitelisted_caller();

        pallet_payment::ComputeHours::<T>::insert(caller.clone(), 100);
        let task_data = get_taskdata(DOCKER_IMAGE_TESTDATA);
        Pallet::<T>::task_scheduler(
            RawOrigin::Signed(caller.clone()).into(),
            TaskKind::OpenInference,
            task_data.clone(),
            None,
            caller.clone(),
            1,
            Some(5),
        )?;
        let task_id = NextTaskId::<T>::get() - 1;

        Pallet::<T>::confirm_task_reception(
            RawOrigin::Signed(caller.clone()).into(),
            task_id,
        )?;
        Pallet::<T>::stop_task_and_vacate_miner(
            RawOrigin::Signed(caller.clone()).into(),
            task_id,
        )?;

        #[block]
        {
            Pallet::<T>::confirm_miner_vacation(
                RawOrigin::Signed(caller.clone()).into(),
                task_id,
            )?;
        }

        assert_eq!(TaskStatus::<T>::get(task_id), Some(TaskStatusType::Vacated));
        Ok(())
    }

    #[benchmark]
    fn set_gatekeeper<T: Config>() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        #[block]
        {
            Pallet::<T>::set_gatekeeper(
                RawOrigin::Root.into(),
                caller.clone(),
            )?;
        }

        assert_eq!(GatekeeperAccount::<T>::get(), Some(caller));
        Ok(())
    }	

	// Defines the benchmark test suite, linking it to the pallet and mock runtime
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
