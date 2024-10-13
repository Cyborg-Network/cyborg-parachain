use codec::{Decode, Encode, EncodeLike};
use cyborg_runtime::apis::{TaskManagementEventsApi, VerifyWorkerRegistration};
use ipfs_api_backend_hyper::TryFromUri;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use log::info;
use sc_client_api::BlockchainEvents;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{sr25519, ConstU32, Pair};
use sp_runtime::traits::Block;
use sp_runtime::BoundedVec;
use std::{env, fs, option, sync::Arc};
use substrate_api_client::ac_primitives::{
	AssetRuntimeConfig, DefaultRuntimeConfig, GenericExtrinsicParams, PlainTip, WithExtrinsicParams,
};
use substrate_api_client::{rpc::TungsteniteRpcClient, Api};
use url::Url;
use worker_spec::gather_worker_spec;

pub mod custom_event_listener;
pub mod donwloade_and_execute_tasks;
pub mod register_worker;
pub mod submit_results;
pub mod worker_spec;

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
	pub creator: String,
	pub worker: (sr25519::Public, u64),
	pub domain: String,
	pub domain_encoded: Vec<u8>,
}

pub struct WorkerConfig {
	domain: BoundedVec<u8, ConstU32<128>>,
	latitude: i32,
	longitude: i32,
	ram: u64,
	storage: u64,
	cpu: u16,
}

pub async fn start_worker<T, U, V, W>(client: Arc<T>)
where
	U: Block,
	V: EncodeLike<sp_runtime::AccountId32>
		+ std::convert::From<sp_core::crypto::CryptoBytes<32, sp_core::sr25519::Sr25519PublicTag>>,
	W: EncodeLike<u64> + std::convert::From<u64>,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: TaskManagementEventsApi<U> + VerifyWorkerRegistration<U, V, W>,
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

	let worker_config = gather_worker_spec(worker_domain);
	let worker_data = bootstrap_worker(client.clone(), api.clone(), worker_config)
		.await
		.unwrap();
	custom_event_listener::event_listener_tester(client, api, ipfs_client, worker_data).await;
}
pub async fn bootstrap_worker<T, U, V, W>(
	client: Arc<T>,
	api: SubstrateClientApi,
	worker_config: WorkerConfig,
) -> option::Option<WorkerData>
where
	U: Block,
	V: EncodeLike<sp_runtime::AccountId32>
		+ std::convert::From<sp_core::crypto::CryptoBytes<32, sp_core::sr25519::Sr25519PublicTag>>,
	W: EncodeLike<u64> + std::convert::From<u64>,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: TaskManagementEventsApi<U> + VerifyWorkerRegistration<U, V, W>,
{
	match fs::read_to_string(CONFIG_FILE_NAME) {
		Err(_e) => {
			info!("worker registation not found, registering worker");
			register_worker::register_worker_on_chain(api, worker_config).await
		}
		Ok(data) => {
			let worker_data: WorkerData = serde_json::from_str(&data).unwrap();
			if verify_worker_registration(client, worker_data.clone()).await {
				Some(worker_data)
			} else {
				register_worker::register_worker_on_chain(api, worker_config).await
			}
		}
	}
}

pub async fn verify_worker_registration<T, U, V, W>(client: Arc<T>, worker_data: WorkerData) -> bool
where
	U: Block,
	V: EncodeLike<sp_runtime::AccountId32>
		+ std::convert::From<sp_core::crypto::CryptoBytes<32, sp_core::sr25519::Sr25519PublicTag>>,
	W: EncodeLike<u64> + std::convert::From<u64>,
	T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
	T::Api: TaskManagementEventsApi<U> + VerifyWorkerRegistration<U, V, W>,
{
	client
		.runtime_api()
		.verify_worker_registration(
			client.info().best_hash,
			(worker_data.worker.0.into(), worker_data.worker.1.into()),
			worker_data.domain_encoded.try_into().unwrap_or_default(),
		)
		.unwrap_or(false)
}
