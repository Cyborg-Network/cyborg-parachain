use sp_runtime::BoundedVec;
use sys_info;

use super::WorkerConfig;

pub fn gather_worker_spec(domain: String) -> WorkerConfig {
	let domain = BoundedVec::try_from(domain.as_bytes().to_vec()).unwrap();
	let ram = sys_info::mem_info().unwrap().total;
	let storage = sys_info::disk_info().unwrap().total;
	let cpu = sys_info::cpu_num().unwrap().try_into().unwrap();
	WorkerConfig {
		domain,
		latitude: 0,
		longitude: 0,
		ram,
		storage,
		cpu,
	}
}
