use frame_support::weights::Weight;
use core::marker::PhantomData;

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_assets::WeightInfo for SubstrateWeight<T> {
    fn create() -> Weight {
        Weight::from_parts(100_000_000, 0)
    }
    
    fn force_create() -> Weight {
        Weight::from_parts(100_000_000, 0)
    }
    
    fn start_destroy() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    
    fn destroy_accounts(c: u32) -> Weight {
        Weight::from_parts(20_000_000, 0).saturating_add(Weight::from_parts(1_000_000, 0).saturating_mul(c as u64))
    }
    
    fn destroy_approvals(a: u32) -> Weight {
        Weight::from_parts(20_000_000, 0).saturating_add(Weight::from_parts(1_000_000, 0).saturating_mul(a as u64))
    }
    
    fn finish_destroy() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }
    
    fn mint() -> Weight {
        Weight::from_parts(80_000_000, 0)
    }
    
    fn burn() -> Weight {
        Weight::from_parts(80_000_000, 0)
    }
    
    fn transfer() -> Weight {
        Weight::from_parts(100_000_000, 0)
    }
    
    fn transfer_keep_alive() -> Weight {
        Weight::from_parts(90_000_000, 0)
    }
    
    fn force_transfer() -> Weight {
        Weight::from_parts(110_000_000, 0)
    }
    
    fn freeze() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn thaw() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn freeze_asset() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn thaw_asset() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn transfer_ownership() -> Weight {
        Weight::from_parts(70_000_000, 0)
    }
    
    fn set_team() -> Weight {
        Weight::from_parts(70_000_000, 0)
    }
    
    fn set_metadata(_name_len: u32, _symbol_len: u32) -> Weight {
        Weight::from_parts(80_000_000, 0)
    }
    
    fn clear_metadata() -> Weight {
        Weight::from_parts(70_000_000, 0)
    }
    
    fn force_set_metadata(_name_len: u32, _symbol_len: u32) -> Weight {
        Weight::from_parts(80_000_000, 0)
    }
    
    fn force_clear_metadata() -> Weight {
        Weight::from_parts(70_000_000, 0)
    }
    
    fn force_asset_status() -> Weight {
        Weight::from_parts(70_000_000, 0)
    }
    
    fn approve_transfer() -> Weight {
        Weight::from_parts(80_000_000, 0)
    }
    
    fn transfer_approved() -> Weight {
        Weight::from_parts(120_000_000, 0)
    }
    
    fn cancel_approval() -> Weight {
        Weight::from_parts(80_000_000, 0)
    }
    
    fn force_cancel_approval() -> Weight {
        Weight::from_parts(90_000_000, 0)
    }
    
    fn set_min_balance() -> Weight {
        Weight::from_parts(70_000_000, 0)
    }
    
    fn touch() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn touch_other() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn refund() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn refund_other() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn block() -> Weight {
        Weight::from_parts(60_000_000, 0)
    }
    
    fn transfer_all() -> Weight {
        Weight::from_parts(110_000_000, 0)
    }
    
}