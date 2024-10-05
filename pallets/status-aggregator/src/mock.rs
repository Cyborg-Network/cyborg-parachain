use frame_support::{derive_impl, parameter_types, weights::constants::RocksDbWeight};
use frame_system::{mocking::MockBlock, GenesisConfig};
use pallet_edge_connect;
use sp_runtime::{
	traits::{ConstU32, ConstU64, ConstU8},
	BuildStorage,
};

// Configure a mock runtime to test the pallet.
#[frame_support::runtime]
mod test_runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

    #[runtime::pallet_index(1)]
	pub type Timestamp = pallet_timestamp;

	#[runtime::pallet_index(2)]
	pub type EdgeConnect = pallet_edge_connect;

	#[runtime::pallet_index(3)]
	pub type StatusAggregator = crate;
}

pub type AccountId = u64;
pub type BlockNumber = u32;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Nonce = u64;
	type Block = MockBlock<Test>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<0>;
	type WeightInfo = ();
}

impl pallet_edge_connect::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
		pub const MaxBlockRangePeriod: BlockNumber = 5u32;
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();

	type MaxBlockRangePeriod = MaxBlockRangePeriod;
	type ThresholdUptimeStatus = ConstU8<75>;
	type MaxAggregateParamLength = ConstU32<10>;
	type WorkerInfoHandler = EdgeConnect;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
