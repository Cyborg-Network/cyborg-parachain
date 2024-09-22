use std::{fs, option, sync::Arc};

use codec::{Decode, Encode};
use cyborg_runtime::apis::TaskManagementEventsApi;
use log::info;
use sc_client_api::BlockchainEvents;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::sr25519;
use sp_runtime::traits::Block;

pub mod custom_event_listener;
pub mod donwloade_and_execute_tasks;
pub mod register_worker;
pub mod submit_results;

pub const CONFIG_FILE_NAME: &str = "registered_worker_config.json";

// datastructure for worker registartion persistence
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode, Serialize, Deserialize)]
pub struct WorkerData {
	creator: String,
	worker: (sr25519::Public, u64),
	domain: String,
	domain_encoded: Vec<u8>,
}

pub async fn start_worker<T, U>(client: Arc<T>)
where
	U: Block,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: TaskManagementEventsApi<U>,
{
	info!("worker_starting");

	let worker_data = bootstrap_worker().await.unwrap();
	custom_event_listener::event_listener_tester(client, worker_data).await;
}
pub async fn bootstrap_worker() -> option::Option<WorkerData> {
	match fs::read_to_string(CONFIG_FILE_NAME) {
		Err(_e) => {
			info!("worker registation not found, registering worker");
			register_worker::register_worker_on_chain().await
		}
		Ok(data) => {
			// TODO: verify worker registration on chain
			let worker_data: WorkerData = serde_json::from_str(&data).unwrap();
			Some(worker_data)
		}
	}
}
