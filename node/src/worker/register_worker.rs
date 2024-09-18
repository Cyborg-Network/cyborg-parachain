use codec::Decode;
use cyborg_runtime as runtime;
use log::{error, info};
use sp_core::crypto::Ss58Codec;
use sp_core::hexdisplay::AsBytesRef;
use sp_core::sr25519;
use sp_core::ConstU32;
use sp_core::Pair;
use sp_runtime::BoundedVec;
use std::env;
use std::io::Write;
use substrate_api_client::ac_node_api::StaticEvent;
use substrate_api_client::ac_primitives::DefaultRuntimeConfig;
use substrate_api_client::SubmitAndWatch;
use substrate_api_client::XtStatus;
use substrate_api_client::{rpc::TungsteniteRpcClient, Api};

use crate::worker::CONFIG_FILE_NAME;

use super::WorkerData;

#[derive(Debug, Clone, PartialEq, Eq, Decode)]
struct EventWorkerRegistered {
	creator: sr25519::Public,
	worker: (sr25519::Public, u64),
	domain: BoundedVec<u8, ConstU32<128>>,
}

impl StaticEvent for EventWorkerRegistered {
	const PALLET: &'static str = "EdgeConnect";
	const EVENT: &'static str = "WorkerRegistered";
}

pub async fn register_worker_on_chain() -> Option<WorkerData> {
	// export CYBORG_WORKER_KEY="e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a" # //Alice
	// export CYBORG_WORKER_DOMAIN="example.com" # replace with your domain
	// run zombienet `zombienet --provider native spawn ./zombienet.toml`

	let worker_key = env::var("CYBORG_WORKER_KEY").expect("CYBORG_WORKER_KEY not set");
	let worker_domain = env::var("CYBORG_WORKER_DOMAIN").expect("CYBORG_WORKER_DOMAIN not set");

	let mut key_32 = [0u8; 32];

	key_32[..].copy_from_slice(&hex::decode(worker_key).unwrap());

	let key = sr25519::Pair::from_seed(&key_32);

	// WARNING: only works on zombienet because of port
	// TODO: get the port from cli arg
	let client = TungsteniteRpcClient::new_with_port("ws://127.0.0.1", 9988, 2).unwrap();

	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).unwrap();

	api.set_signer(key.clone().into());

	let domain = BoundedVec::try_from(worker_domain.as_bytes().to_vec()).unwrap();

	let register_call =
		runtime::RuntimeCall::EdgeConnect(runtime::pallet_edge_connect::Call::register_worker {
			domain,
		});

	let tr_tx = api.compose_extrinsic_offline(register_call, 0);
	info!("{:?}", &tr_tx);

	let ext_response = api.submit_and_watch_extrinsic_until(tr_tx, XtStatus::InBlock);
	println!("{:?}", &ext_response);

	match ext_response {
		Ok(ext_response_succ) => {
			println!("{:?}", &ext_response_succ);
			println!("Worker registered successfully ✅✅✅✅✅✅✅✅✅✅✅✅✅");
			println!("Events: {:?}", ext_response_succ.events);

			for event in ext_response_succ.events.unwrap() {
				if event.pallet_name() == "EdgeConnect" {
					let decoded_event = event.as_event::<EventWorkerRegistered>();
					info!("{:?}", &decoded_event);
					match decoded_event {
						Err(_) | Ok(None) => {
							println!("❌Event not decoded properly❌");
						}
						Ok(Some(registration_event)) => {
							return worker_retain_after_restart(registration_event);
						}
					}
				}
			}
			None
		}
		Err(e) => {
			println!("Somethign went wrong while registering worker ❌❌❌❌❌❌❌❌❌❌❌❌❌❌❌❌");
			error!("{:?}", e);
			println!("RESTART The worker node with proper environment variables");
			None
		}
	}
}

fn worker_retain_after_restart(reg_event: EventWorkerRegistered) -> Option<WorkerData> {
	let registered_worker_data = WorkerData {
		creator: reg_event.creator.to_ss58check(),
		worker: reg_event.worker,
		domain: String::from_utf8_lossy(reg_event.domain.as_bytes_ref()).to_string(),
		domain_encoded: reg_event.domain.into(),
	};

	let registered_worker_json = serde_json::to_string_pretty(&registered_worker_data);
	info!("{:?}", &registered_worker_json);

	use std::{fs::File, path::Path};

	let config_path = Path::new(CONFIG_FILE_NAME);
	match File::create(config_path) {
		Err(e) => {
			error!("{}", e);
			None
		}
		Ok(mut created_file) => {
			created_file
				.write_all(registered_worker_json.unwrap().as_bytes())
				.unwrap_or_else(|_| panic!("Unable to write file : {:?}", config_path.to_str()));
			println!(
				"✅✅Saved worker registration data to file: {:?}✅✅ ",
				config_path.to_str()
			);
			Some(registered_worker_data)
		}
	}
}
