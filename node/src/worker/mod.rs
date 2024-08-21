use std::sync::Arc;

use cyborg_runtime::apis::RuntimeApi;
use sc_client_api::BlockchainEvents;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block;

pub mod custom_event_listener;

pub async fn start_worker<T, U>(client: Arc<T>) -> ()
where
	U: Block,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: sp_block_builder::BlockBuilder<U> + sp_api::Core<U>,
	// T::Api: cyborg_runtime::apis::RuntimeApi<U>,
{
	dbg!("============worker_starting============");

	custom_event_listener::event_listener_tester(client).await;
}
