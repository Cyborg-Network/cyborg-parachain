use cyborg_runtime as runtime;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use log::{error, info};
use sp_core::{sr25519, Pair, H256};
use sp_runtime::BoundedVec;
use std::{
	env,
	process::{Child, ChildStdout},
};
use substrate_api_client::{
	ac_primitives::DefaultRuntimeConfig, rpc::TungsteniteRpcClient, Api, SubmitAndWatch, XtStatus,
};

pub async fn submit_result_onchain(mut result: Child, task_id: u64) {
	dbg!(&result);
	let result_raw_data = result.stdout.take().unwrap();
	dbg!(&result_raw_data);

	let hash = publish_on_ipfs(result_raw_data).await;
	submit_to_chain(hash, task_id).await;
}

pub async fn publish_on_ipfs(result: ChildStdout) -> String {
	let ipfs_client = IpfsClient::default();

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

pub async fn submit_to_chain(result: String, task_id: u64) {
	let client = TungsteniteRpcClient::new_with_port("ws://127.0.0.1", 9988, 2).unwrap();

	let worker_key = env::var("CYBORG_WORKER_KEY").expect("CYBORG_WORKER_KEY not set");

	let mut key_32 = [0u8; 32];

	key_32[..].copy_from_slice(&hex::decode(worker_key).unwrap());

	let key = sr25519::Pair::from_seed(&key_32);
	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).unwrap();

	api.set_signer(key.clone().into());

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