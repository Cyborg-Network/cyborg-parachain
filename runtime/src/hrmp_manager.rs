use frame_support::{
    traits::{HrmpChannelManager, OriginTrait},
    dispatch::DispatchResult,
};
use polkadot_parachain_primitives::primitives::Id as ParaId;
use polkadot_primitives::HrmpChannelId;
use sp_runtime::traits::AccountIdConversion;
use xcm::latest::MultiLocation;

use crate::{AccountId, Runtime, RuntimeOrigin};

pub struct HrmpManager;

impl HrmpChannelManager for HrmpManager {
    type AccountId = AccountId;

    fn init_open_channel(
        sender: ParaId,
        recipient: ParaId,
        proposed_max_capacity: u32,
        proposed_max_message_size: u32,
    ) -> DispatchResult {
        // Convert para IDs to account IDs for the HRMP pallet
        let sender_account: AccountId = sender.into_account_truncating();
        let recipient_account: AccountId = recipient.into_account_truncating();
        
        // Create the HRMP channel ID
        let channel_id = HrmpChannelId { sender, recipient };
        
        // Call into the HRMP pallet to initiate channel opening
        pallet_hrmp::Pallet::<Runtime>::init_open_channel(
            RuntimeOrigin::signed(sender_account),
            recipient,
            proposed_max_capacity,
            proposed_max_message_size,
        )?;
        
        Ok(())
    }

    fn accept_open_channel(recipient: ParaId) -> DispatchResult {
        let recipient_account: AccountId = recipient.into_account_truncating();
        
        // Call into the HRMP pallet to accept the channel
        pallet_hrmp::Pallet::<Runtime>::accept_open_channel(
            RuntimeOrigin::signed(recipient_account),
            recipient,
        )?;
        
        Ok(())
    }

    fn close_channel(channel_id: HrmpChannelId) -> DispatchResult {
        // For closing, we need to determine which party is initiating the closure
        // We'll use the sender for simplicity, 
        let closer_account: AccountId = channel_id.sender.into_account_truncating();
        
        pallet_hrmp::Pallet::<Runtime>::close_channel(
            RuntimeOrigin::signed(closer_account),
            channel_id,
        )?;
        
        Ok(())
    }
}

// Additional helper functions for HRMP management
impl HrmpManager {
    /// Force open an HRMP channel (governance function)
    pub fn force_open_channel(
        sender: ParaId,
        recipient: ParaId,
        max_capacity: u32,
        max_message_size: u32,
    ) -> DispatchResult {
        let root_origin = RuntimeOrigin::root();
        
        pallet_hrmp::Pallet::<Runtime>::force_open_hrmp_channel(
            root_origin,
            sender,
            recipient,
            max_capacity,
            max_message_size,
        )?;
        
        Ok(())
    }
    
    /// Check if an HRMP channel exists
    pub fn channel_exists(sender: ParaId, recipient: ParaId) -> bool {
        let channel_id = HrmpChannelId { sender, recipient };
        pallet_hrmp::HrmpChannels::<Runtime>::contains_key(channel_id)
    }
    
    /// Get channel information
    pub fn get_channel(sender: ParaId, recipient: ParaId) -> Option<pallet_hrmp::HrmpChannel> {
        let channel_id = HrmpChannelId { sender, recipient };
        pallet_hrmp::HrmpChannels::<Runtime>::get(channel_id)
    }
}