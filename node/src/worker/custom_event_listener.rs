use std::sync::Arc;

use cyborg_runtime::apis::TaskManagementEventsApi;
use futures::StreamExt;
use sc_client_api::BlockchainEvents;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::hexdisplay::AsBytesRef;
use sp_runtime::traits::Block;

use super::WorkerData;

pub async fn event_listener_tester<T, U>(client: Arc<T>, worker_data: WorkerData)
where
	U: Block,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: TaskManagementEventsApi<U>,
{
	dbg!("============ event_listener_tester ============");

	let mut blocks = client.every_import_notification_stream();

	while let Some(block_import_notification) = blocks.next().await {
		dbg!("block_import_notification");

		let block_hash = block_import_notification.hash;

		match client.runtime_api().get_recent_events(block_hash) {
			Ok(event_vec) => {
				dbg!(&event_vec);
				for event in event_vec {
					match event {
						cyborg_runtime::pallet_task_management::Event::TaskScheduled {
							assigned_worker,
							task_owner,
							task_id,
							task,
							..
						} => {
							if assigned_worker == (worker_data.worker.0.into(), worker_data.worker.1) {
								dbg!(task_id, task_owner);
								let docker_image_name = String::from_utf8_lossy(task.as_bytes_ref());
								dbg!(docker_image_name);
							}
						}

						_ => {}
					}
				}
			}
			Err(e) => {
				dbg!(e);
			}
		}
	}
}
