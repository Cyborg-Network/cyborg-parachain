use futures::TryStreamExt;
use ipfs_api_backend_hyper::{self, IpfsApi, IpfsClient};
use log::{error, info};
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

pub const WORK_FILES_DIR: &str = "tasks_binary";

pub async fn download_and_execute_work_package(
	ipfs_hash: &str,
	ipfs_client: &IpfsClient,
) -> Option<Result<std::process::Child, std::io::Error>> {
	info!("ipfs_hash: {}", ipfs_hash);
	info!("============ download_file ============");
	// TODO: validate its a valid ipfs hash
	match ipfs_client
		.cat(&format!("/ipfs/{ipfs_hash}"))
		.map_ok(|chunk| chunk.to_vec())
		.try_concat()
		.await
	{
		Err(e) => {
			error!("{}", e);
			None
		}
		Ok(data) => {
			info!("got data from ipfs with length of {}", &data.len());
			// TODO: check and create directory
			let file_path = format!("./{WORK_FILES_DIR}/{ipfs_hash}");

			let mut file = File::create(&file_path).unwrap();
			let mut perms = file.metadata().unwrap().permissions();
			perms.set_mode(perms.mode() | 0o111);

			file.write_all(&data).unwrap();

			file.set_permissions(perms).unwrap();
			Some(Command::new(file_path).stdout(Stdio::piped()).spawn())
		}
	}
}
