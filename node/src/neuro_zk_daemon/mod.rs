/*use async_std::{net::TcpListener, prelude::*, task};
use async_h1::server;
use http_types::{Request, Response, StatusCode};
use sc_client_api::HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block;
use std::sync::Arc;

type ParachainClient = impl ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static;

pub fn start_daemon<T>(client: Arc<T>) -> impl futures::Future<Output = ()> + Send + 'static 
where
    T: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    T::Api: NeuroZkStorageApi<Block>,
{
	let client = client.clone();

    async move {
        task::spawn(async {
            let listener = TcpListener::bind("0.0.0.0:6666")
                .await
                .expect("Failed to bind port 6666");
            println!("Listening on http://0.0.0.0:6666");

            while let Ok((stream, _peer_addr)) = listener.accept().await {
				let client = client.clone();
                task::spawn(handle_connection(stream));
            }
        });
    }
}

async fn handle_connection(stream: async_std::net::TcpStream) {
    if let Err(err) = server::accept(stream, handle_request).await {
        eprintln!("Request handling error: {:?}", err);
    }
}

async fn handle_request(req: Request) -> http_types::Result<Response> {
    if req.method() == http_types::Method::Get && req.url().path() == "/verify" {
        println!("Received GET /verify");
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body("Verification complete");
        Ok(resp)
    } else {
        let mut resp = Response::new(StatusCode::NotFound);
        resp.set_body("Not Found");
        Ok(resp)
    }
}    				



fn fetch_proof_data(client : Arc<ParachainClient>) {
	let best_hash = client.info().best_hash;
    
	let tasks: Vec<u8> = client
        .runtime_api()
        .get_something(best_hash /*, your args */);

	// iterate over tasks and make another runtime call for each, collecting the results into a new vector after decoding them

	// return the vector
}

fn verify_proofs(proof_data: Vec<ProofDataStruct>) {
	//
}
*/

use async_std::{net::TcpListener, task};
use async_h1::server;
use http_types::{Request, Response, StatusCode};
use sc_client_api::HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block;
use std::sync::Arc;

use cyborg_runtime::apis::NeuroZkStorageApi;
use cyborg_primitives::proof::NeuroZkTaskInfo;

pub fn start_daemon<B, T>(client: Arc<T>) -> impl futures::Future<Output = ()> + Send + 'static 
where
    B: Block,
    T: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    T::Api: NeuroZkStorageApi<B>,
{
    async move {
        let listener = TcpListener::bind("0.0.0.0:6666")
            .await
            .expect("Failed to bind port 6666");
        println!("Listening on http://0.0.0.0:6666");

        while let Ok((stream, _peer_addr)) = listener.accept().await {
            let client = client.clone();
            task::spawn(async move {
                if let Err(err) = server::accept(stream, move |req| {
                    handle_request(req, client.clone())
                }).await {
                    eprintln!("Request handling error: {:?}", err);
                }
            });
        }
    }
}

async fn handle_request<B, T>(req: Request, client: Arc<T>) -> http_types::Result<Response>
where
    B: Block, 
    T: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    T::Api: NeuroZkStorageApi<B>,
{
    if req.method() == http_types::Method::Get && req.url().path() == "/verify" {
        println!("Received GET /verify");

        let proof_data = match fetch_proof_data(client).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to fetch proof data: {:?}", e);
                let mut res = Response::new(StatusCode::InternalServerError);
                res.set_body("Internal Server Error");
                
                return Ok(res);
            }
        };

        verify_proofs(&proof_data); // currently dummy

        let mut res = Response::new(StatusCode::Ok);
        res.set_body("Verification complete");
        Ok(res)
    } else {
        let mut res = Response::new(StatusCode::NotFound);
        res.set_body("Not Found");
        Ok(res)
    }
}

async fn fetch_proof_data<B, T>(client: Arc<T>) -> Result<Vec<NeuroZkTaskInfo>, sp_api::ApiError>
where
    B: Block,
    T: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    T::Api: NeuroZkStorageApi<B>,
{
    let best_hash = client.info().best_hash;

    let task_ids = client.runtime_api().retrieve_current_tasks(best_hash)?;
    println!("Found {} task IDs", task_ids.len());

    let mut proofs = Vec::new();

    for task_id in task_ids {
        let maybe_nzk_info = client.runtime_api().retrieve_verification_data(best_hash, task_id)?;
        if let Some(nzk_info) = maybe_nzk_info {
            proofs.push(nzk_info);
        }
    }

    Ok(proofs)
}

fn verify_proofs(proof_data: &[NeuroZkTaskInfo]) {
    for proof in proof_data {
        println!("Verifying proof...");
        // Your verification logic here
    }
}
