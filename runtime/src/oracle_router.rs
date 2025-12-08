use super::{AccountId, Runtime};
pub use cyborg_primitives::oracle::{OracleKey, OracleValue};
use orml_traits::OnNewData;
pub use pallet_status_aggregator;

/// The oracle router decides which pallet to route the incoming data to, based on the key.
pub struct OracleRouter;

impl OnNewData<AccountId, OracleKey, OracleValue> for OracleRouter {
	fn on_new_data(who: &AccountId, key: &OracleKey, value: &OracleValue) {
		match (key, value) {
			(&OracleKey::Miner(ref inner_key), &OracleValue::MinerStatus(ref process_status)) => {
				pallet_status_aggregator::Pallet::<Runtime>::on_new_data(who, inner_key, process_status);
			}
			_ => {
				log::warn!("Mismatched OracleKey and OracleValue types!");
			}
		}
	}
}
