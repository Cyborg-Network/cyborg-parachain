use cyborg_runtime as runtime;
use sp_core::sr25519;
use sp_core::Pair;
use sp_runtime::BoundedVec;
use std::env;
use substrate_api_client::ac_primitives::DefaultRuntimeConfig;
use substrate_api_client::SubmitAndWatch;
use substrate_api_client::XtStatus;
use substrate_api_client::{rpc::TungsteniteRpcClient, Api};

pub async fn register_worker_on_chain() {
	// TODO: get all the extrinsic params from environment variables (domain)

	dbg!("============register_worker_on_chain============");

	// export CYBORG_WORKER_KEY="e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a" #// Alice
	// run zombienet `zombienet --provider native spawn ./zombienet.toml`

	let worker_key = env::var("CYBORG_WORKER_KEY").expect("CYBORG_WORKER_KEY not set");

	let mut key_32 = [0u8; 32];

	key_32[..].copy_from_slice(&hex::decode(worker_key).unwrap());

	let key = sr25519::Pair::from_seed(&key_32);

	// WARNING: only works on zombienet because of port
	// TODO: get the port from cli arg
	let client = TungsteniteRpcClient::new_with_port("ws://127.0.0.1", 9988, 2).unwrap();

	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).unwrap();

	dbg!(key.public());
	api.set_signer(key.clone().into());

	let domain = BoundedVec::new();

	let register_call =
		runtime::RuntimeCall::EdgeConnect(runtime::pallet_edge_connect::Call::register_worker {
			ip: None,
			domain: Some(domain),
		});

	let tr_tx = api.compose_extrinsic_offline(register_call, 0);
	dbg!(&tr_tx);

	let ext_response = api.submit_and_watch_extrinsic_until(tr_tx, XtStatus::InBlock);
	dbg!(&ext_response);

	match ext_response {
		Ok(_) => {
			println!("Worker registered successfully ✅✅✅✅✅✅✅✅✅✅✅✅✅");
		}
		Err(e) => {
			println!("Somethign went wrong while registering worker ❌❌❌❌❌❌❌❌❌❌❌❌❌❌❌❌");
			dbg!(e);
			println!("RESTART The worker node with proper environment variables");
		}
	}
}
