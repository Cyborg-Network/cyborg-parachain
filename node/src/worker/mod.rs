use codec::{Decode, Encode};
use cyborg_runtime::apis::TaskManagementEventsApi;
use ipfs_api_backend_hyper::TryFromUri;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use log::info;
use sc_client_api::BlockchainEvents;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{sr25519, Pair};
use sp_runtime::traits::Block;
use std::{env, fs, option, sync::Arc};
use substrate_api_client::ac_primitives::{
	AssetRuntimeConfig, DefaultRuntimeConfig, GenericExtrinsicParams, PlainTip, WithExtrinsicParams,
};
use substrate_api_client::{rpc::TungsteniteRpcClient, Api};
use url::Url;

pub mod custom_event_listener;
pub mod donwloade_and_execute_tasks;
pub mod register_worker;
pub mod submit_results;

pub const CONFIG_FILE_NAME: &str = "registered_worker_config.json";

pub const IPFS_DEFAULT_URI: &str =
	"https://8be9886d720942e0be9c10bc4351e9dd:ea84a88bd688458188735bff8c576e90@ipfs.infura.io:5001/api/v0";

pub type SubstrateClientApi = Api<
	WithExtrinsicParams<
		AssetRuntimeConfig,
		GenericExtrinsicParams<AssetRuntimeConfig, PlainTip<u128>>,
	>,
	TungsteniteRpcClient,
>;

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

	// export CYBORG_WORKER_KEY="e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a" # //Alice
	// export CYBORG_WORKER_DOMAIN="example.com" # replace with your domain

	// IPFS="https://<infura_project_api_key>:<infura_api_secret_key>@ipfs.infura.io:5001/api/v0"
	// run zombienet `zombienet --provider native spawn ./zombienet.toml`

	let worker_key = env::var("CYBORG_WORKER_KEY").expect("CYBORG_WORKER_KEY not set");
	let worker_domain = env::var("CYBORG_WORKER_DOMAIN").expect("CYBORG_WORKER_DOMAIN not set");
	let ipfs_credentials = env::var("IPFS");

	let ipfs_url = ipfs_credentials
		.ok()
		.and_then(|creds| creds.parse::<Url>().ok())
		.unwrap_or_else(|| {
			IPFS_DEFAULT_URI
				.parse::<Url>()
				.expect("Invalid default IPFS URI")
		});

	let mut key_32 = [0u8; 32];

	key_32[..].copy_from_slice(&hex::decode(worker_key).unwrap());

	let key = sr25519::Pair::from_seed(&key_32);

	// WARNING: only works on zombienet because of port
	// TODO: get the port from cli arg
	let api_client = TungsteniteRpcClient::new_with_port("ws://127.0.0.1", 9988, 2).unwrap();

	let mut api = Api::<DefaultRuntimeConfig, _>::new(api_client).unwrap();

	api.set_signer(key.clone().into());

	let ipfs_client = IpfsClient::build_with_base_uri(ipfs_url.to_string().parse().unwrap())
		.with_credentials(ipfs_url.username(), ipfs_url.password().unwrap());

	let version_out = ipfs_client.version().await;
	info!("version_out: {:?}", &version_out);

	let worker_data = bootstrap_worker(api.clone(), worker_domain).await.unwrap();
	custom_event_listener::event_listener_tester(client, api, ipfs_client, worker_data).await;
}
pub async fn bootstrap_worker(
	api: SubstrateClientApi,
	worker_domain: String,
) -> option::Option<WorkerData> {
	match fs::read_to_string(CONFIG_FILE_NAME) {
		Err(_e) => {
			info!("worker registation not found, registering worker");
			register_worker::register_worker_on_chain(api, worker_domain).await
		}
		Ok(data) => {
			// TODO: verify worker registration on chain
			let worker_data: WorkerData = serde_json::from_str(&data).unwrap();
			Some(worker_data)
		}
	}
}
