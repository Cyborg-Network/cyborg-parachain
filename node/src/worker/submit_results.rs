use cyborg_runtime as runtime;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use log::{error, info};
use sp_core::H256;
use sp_runtime::BoundedVec;
use std::process::{Child, ChildStdout};
use substrate_api_client::{SubmitAndWatch, XtStatus};

use super::SubstrateClientApi;

pub async fn submit_result_onchain(
	api: &SubstrateClientApi,
	ipfs_client: &IpfsClient,
	mut result: Child,
	task_id: u64,
) {
	dbg!(&result);
	let result_raw_data = result.stdout.take().unwrap();
	dbg!(&result_raw_data);

	let hash = publish_on_ipfs(result_raw_data, ipfs_client).await;
	submit_to_chain(api, hash, task_id).await;
}

pub async fn publish_on_ipfs(result: ChildStdout, ipfs_client: &IpfsClient) -> String {
	let ipfs_res = ipfs_client.add(result).await;
	if let Ok(ipfs_res) = ipfs_res {
		let hash = ipfs_res.hash;
		dbg!(&hash);
		hash
	} else {
		error!("Failed to publish on IPFS");
		String::new()
	}
}

pub async fn submit_to_chain(api: &SubstrateClientApi, result: String, task_id: u64) {
	let result = BoundedVec::try_from(result.as_bytes().to_vec()).unwrap();

	let completed_hash = H256::random();

	let register_call = runtime::RuntimeCall::TaskManagement(
		runtime::pallet_task_management::Call::submit_completed_task {
			task_id,
			completed_hash,
			result,
		},
	);

	dbg!(&register_call);

	let tr_tx = api.compose_extrinsic_offline(register_call, api.get_nonce().unwrap());
	info!("{:?}", &tr_tx);

	let ext_response = api.submit_and_watch_extrinsic_until(tr_tx, XtStatus::InBlock);
	if let Ok(ext_response) = ext_response {
		info!("Task submited successfully ✅✅✅✅✅✅✅✅✅✅✅✅✅");
		info!("{:?}", ext_response);
	} else {
		error!("❌ Something went wrong on result submission ❌");

		info!("{:?}", &ext_response);
	}
}
