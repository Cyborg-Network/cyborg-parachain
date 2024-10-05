use sp_runtime::BoundedVec;

use super::WorkerConfig;

pub fn gather_worker_spec() -> WorkerConfig {
	WorkerConfig {
		domain: BoundedVec::new(),
		latitude: 0,
		longitude: 0,
		ram: 0,
		storage: 0,
		cpu: 0,
	}
}
