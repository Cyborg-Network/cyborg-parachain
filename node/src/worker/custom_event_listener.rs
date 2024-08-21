use std::sync::Arc;

use cyborg_runtime;
use futures::StreamExt;
use sc_client_api::BlockchainEvents;
use sp_api::Core;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block;

pub async fn event_listener_tester<T, U>(client: Arc<T>)
where
	U: Block,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: sp_block_builder::BlockBuilder<U> + sp_api::Core<U>,
	// T::Api: cyborg_runtime::apis::RuntimeApi,
{
	dbg!("============ event_listener_tester ============");

	let mut blocks = client.every_import_notification_stream();

	while let Some(block_import_notification) = blocks.next().await {
		dbg!(&block_import_notification);

		let block_hash = block_import_notification.hash;

		match client.runtime_api().version(block_hash) {
			Ok(block) => {
				dbg!(block);
			}
			Err(e) => {
				dbg!(e);
			}
		}
	}
}
