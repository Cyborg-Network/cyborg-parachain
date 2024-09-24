use codec::Decode;
use cyborg_runtime as runtime;
use log::{error, info};
use sp_core::crypto::Ss58Codec;
use sp_core::hexdisplay::AsBytesRef;
use sp_core::sr25519;
use sp_core::ConstU32;
use sp_runtime::BoundedVec;
use std::io::Write;
use substrate_api_client::ac_node_api::StaticEvent;
use substrate_api_client::SubmitAndWatch;
use substrate_api_client::XtStatus;

use crate::worker::CONFIG_FILE_NAME;

use super::SubstrateClientApi;
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

pub async fn register_worker_on_chain(
	api: SubstrateClientApi,
	worker_domain: String,
) -> Option<WorkerData> {
	let domain = BoundedVec::try_from(worker_domain.as_bytes().to_vec()).unwrap();

	let register_call =
		runtime::RuntimeCall::EdgeConnect(runtime::pallet_edge_connect::Call::register_worker {
			domain,
		});

	let tr_tx = api.compose_extrinsic_offline(register_call, api.get_nonce().unwrap());
	info!("{:?}", &tr_tx);

	let ext_response = api.submit_and_watch_extrinsic_until(tr_tx, XtStatus::InBlock);
	info!("{:?}", &ext_response);

	match ext_response {
		Ok(ext_response_succ) => {
			info!("{:?}", &ext_response_succ);
			info!("Worker registered successfully ✅✅✅✅✅✅✅✅✅✅✅✅✅");
			info!("Events: {:?}", ext_response_succ.events);

			for event in ext_response_succ.events.unwrap() {
				if event.pallet_name() == "EdgeConnect" {
					let decoded_event = event.as_event::<EventWorkerRegistered>();
					info!("{:?}", &decoded_event);
					match decoded_event {
						Err(_) | Ok(None) => {
							error!("❌Event not decoded properly❌");
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
			error!("Somethign went wrong while registering worker ❌❌❌❌❌❌❌❌❌❌❌❌❌❌❌❌");
			error!("{:?}", e);
			error!("RESTART The worker node with proper environment variables");
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
			info!(
				"✅✅Saved worker registration data to file: {:?}✅✅ ",
				config_path.to_str()
			);
			Some(registered_worker_data)
		}
	}
}
