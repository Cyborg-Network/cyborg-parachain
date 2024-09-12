use std::sync::Arc;

use codec::{Decode, Encode};
use cyborg_runtime::apis::TaskManagementEventsApi;
use sc_client_api::BlockchainEvents;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::sr25519;
use sp_runtime::traits::Block;

pub mod custom_event_listener;
pub mod register_worker;

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
	dbg!("============worker_starting============");

	register_worker::register_worker_on_chain().await;
	custom_event_listener::event_listener_tester(client).await;
}
