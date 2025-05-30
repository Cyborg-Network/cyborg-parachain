pub use cyborg_primitives::{
	oracle::{OracleKey, OracleValue},
};
pub use pallet_neuro_zk;
pub use pallet_status_aggregator;
use super::{ AccountId, Runtime };
use orml_traits::OnNewData;

/// The oracle router decides which pallet to route the incoming data to, based on the key.
pub struct OracleRouter;

impl OnNewData<
    AccountId,
    OracleKey<AccountId>,
    OracleValue,
> for OracleRouter {
    fn on_new_data(
        who: &AccountId,
        key: &OracleKey<AccountId>,
        value: &OracleValue,
    ) {
        match ( key, value ) {
            (&OracleKey::Miner(ref inner_key), &OracleValue::MinerStatus(ref process_status)) => {
                pallet_status_aggregator::Pallet::<Runtime>::on_new_data(who, inner_key, process_status);
            },
            (&OracleKey::NzkProofResult(ref inner_key), &OracleValue::ZkProofResult(ref result)) => {
                pallet_neuro_zk::Pallet::<Runtime>::on_new_data(who, inner_key, result);
            },
			_ => {
                log::warn!("Mismatched OracleKey and OracleValue types!");
            }
        }
    }
}