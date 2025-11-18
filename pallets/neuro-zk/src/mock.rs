pub use crate as pallet_neuro_zk;
use frame_support::traits::EnsureOriginWithArg;
use frame_support::{derive_impl, parameter_types, weights::constants::RocksDbWeight};
use frame_system::EnsureRoot;
use frame_system::EnsureRootWithSuccess;
use frame_system::{mocking::MockBlock, GenesisConfig};
use pallet_edge_connect;
use pallet_payment;
use pallet_task_management;
use sp_runtime::{
	traits::{ConstU32, ConstU64, ConstU8},
	BuildStorage,
};

pub type AssetId = u32;
// Configure a mock runtime to test the pallet.
#[frame_support::runtime]
mod test_runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeError,
		RuntimeEvent,
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
	pub type TaskManagementModule = pallet_task_management;

	#[runtime::pallet_index(4)]
	pub type PaymentModule = pallet_payment;

	#[runtime::pallet_index(5)]
	pub type NeuroZk = pallet_neuro_zk;

	#[runtime::pallet_index(6)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(7)]
	pub type Assets = pallet_assets;
}

pub type AccountId = u64;
pub type Balance = u128;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = MockBlock<Test>;
	type Nonce = u64;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
	type AccountData = pallet_balances::AccountData<u128>;
}

parameter_types! {
	pub const AssetDeposit: u128 = 100;
	pub const AssetAccountDeposit: u128 = 10;
	pub const ApprovalDeposit: u128 = 1;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: u128 = 10;
	pub const MetadataDepositPerByte: u128 = 1;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = u32;
	type AssetIdParameter = codec::Compact<u32>;
	type Currency = Balances;
	type CreateOrigin = EnsureRootWithSuccess<AccountId, ConstU64<12345>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
	type CallbackHandle = ();
	type Holder = ();
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

impl pallet_task_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type TaskConfirmationTimeout = TaskConfirmationTimeout;
}

parameter_types! {
	pub const TaskConfirmationTimeout: u64 = 75; // ~7.5 minutes at 6s/block
}

impl pallet_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TreasuryAccount = TreasuryAccount;
	type WeightInfo = ();
	type MaxKycHashLength = ConstU32<64>;
	type MaxPaymentIdLength = MaxPaymentIdLength;
	type MaxUserIdLength = MaxUserIdLength;

	type AssetRegistry = Assets;
    type AssetId = AssetId;
    type AssetBalance = Balance;
    type AssetAuthority = EnsureRootWithAccount;
}

pub struct EnsureRootWithAccount;

impl EnsureOriginWithArg<RuntimeOrigin, u32> for EnsureRootWithAccount {
	type Success = AccountId;

	fn try_origin(origin: RuntimeOrigin, _asset_id: &u32) -> Result<Self::Success, RuntimeOrigin> {
		EnsureRoot::<AccountId>::try_origin(origin.clone(), &())?;
		Ok(TreasuryAccount::get())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(_asset_id: &u32) -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}
parameter_types! {
	pub const TreasuryAccount: u64 = 999;
	pub const ExistentialDeposit: u128 = 10;
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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}
