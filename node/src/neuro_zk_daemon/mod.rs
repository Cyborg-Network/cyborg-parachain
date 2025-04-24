use codec::{Decode, Encode, EncodeLike};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block;
use std::sync::Arc;
use std::thread;
//use sc_client_api::BlockchainEvents;
use cyborg_runtime::apis::TaskManagementEventsApi;

use tiny_http::{Request, Response, Server};

pub fn start_daemon(//client: Arc<T>
) -> impl futures::Future<Output = ()> + Send + 'static
where
		//   U: Block,
		//  V: EncodeLike<sp_runtime::AccountId32>
		// + std::convert::From<sp_core::crypto::CryptoBytes<32, sp_core::sr25519::Sr25519PublicTag>>,
		//  W: EncodeLike<u64> + std::convert::From<u64>,
		//  T: ProvideRuntimeApi<U> + HeaderBackend<U> + BlockchainEvents<U>,
		//  T::Api: TaskManagementEventsApi<U>,
{
	async move {
		thread::spawn(move || {
			let server = Server::http("0.0.0.0:6666").unwrap();

			for request in server.incoming_requests() {
				if request.method() != &tiny_http::Method::Get {
					let _ =
						request.respond(Response::from_string("Method Not Allowed").with_status_code(405));
					continue;
				}

				let url = request.url(); // e.g. "/verify/12345"
				if let Some(id) = url.strip_prefix("/verify/") {
					// Call into the runtime API using the Substrate client

					let response = format!("ID {}", id);
					let _ = request.respond(Response::from_string(response));
				} else {
					let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
				}
			}
		});
	}
}
