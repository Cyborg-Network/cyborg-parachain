use frame_support::weights::Weight;
use core::marker::PhantomData;

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_treasury::WeightInfo for SubstrateWeight<T> {
    fn spend() -> Weight {
        Weight::from_parts(100_000_000, 0)
    }
    fn spend_local() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    fn on_initialize_proposals(_p: u32) -> Weight {
        Weight::from_parts(10_000_000, 0)
    }
    fn payout() -> Weight {
        Weight::from_parts(75_000_000, 0)
    }
    fn check_status() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }
    fn void_spend() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    fn remove_approval() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }
}