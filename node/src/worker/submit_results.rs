use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use std::process::Child;

pub async fn submit_result_onchain(mut result: Child) {
	dbg!(&result);
	let ipfs_client = IpfsClient::default();
	let result_raw_data = result.stdout.take().unwrap();
	dbg!(&result_raw_data);

	let ipfs_hash_of_result_data = ipfs_client.add(result_raw_data).await;
	if let Ok(ipfs_res) = ipfs_hash_of_result_data {
		let hash = ipfs_res.hash;
		dbg!(hash);
	}
}
