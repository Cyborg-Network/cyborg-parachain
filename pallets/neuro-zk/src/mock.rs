pub use crate as pallet_neuro_zk;
use frame_support::{derive_impl, parameter_types, weights::constants::RocksDbWeight};
use frame_system::{mocking::MockBlock, GenesisConfig};
use pallet_edge_connect;
use pallet_payment;
use pallet_task_management;
use sp_runtime::{
	traits::{ConstU32, ConstU64, ConstU8},
	BuildStorage,
};
use cyborg_primitives::constants::{DAYS, HOURS};


pub type Balance = u128;

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
	pub type EdgeConnectModule = pallet_edge_connect;

    #[runtime::pallet_index(3)]
    pub type Balances = pallet_balances;

	#[runtime::pallet_index(4)]
	pub type TaskManagementModule = pallet_task_management;

	#[runtime::pallet_index(5)]
	pub type PaymentModule = pallet_payment;

	#[runtime::pallet_index(6)]
	pub type NeuroZk = pallet_neuro_zk;
}

pub type AccountId = u128;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = MockBlock<Test>;
	type Nonce = u64;
    type AccountId = AccountId;
    type AccountData = pallet_balances::AccountData<Balance>;
    type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
}

parameter_types! {
		pub const MaxBlockRangePeriod: u32 = 5;
		pub const MaxPaymentIdLength: u32 = 128;
		pub const MaxUserIdLength: u32 = 128;
}

impl pallet_neuro_zk::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type AcceptanceThreshold = ConstU8<75>;
	type AggregateLength = ConstU32<5>;
	type NzkTaskInfoHandler = TaskManagementModule;
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

impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = ();
    type WeightInfo = ();
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
    type DoneSlashHandler = ();
}

impl pallet_task_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type TaskConfirmationTimeout = TaskConfirmationTimeout;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 10;
	pub const TaskConfirmationTimeout: u64 = 75; // ~7.5 minutes at 6s/block
    pub const OnDemandRate: Balance = 2;
    pub const SubscriptionRate: Balance = 10; 
}

impl pallet_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type MaxKycHashLength = ConstU32<64>;
	type MaxPaymentIdLength = MaxPaymentIdLength;
	type MaxUserIdLength = MaxUserIdLength;
    type SubscriptionPeriod = ConstU64<{ 30 * DAYS }>;
    type OnDemandPeriod = ConstU64<{ HOURS }>;
    type GracePeriod = ConstU64<{ 4 * DAYS }>;
    type OnDemandRate = OnDemandRate;
    type SubscriptionRate = SubscriptionRate;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
