pub use crate as pallet_payment;
use frame_support::{derive_impl, parameter_types, weights::constants::RocksDbWeight};
use frame_system::EnsureRoot;
use frame_system::EnsureRootWithSuccess;
use frame_system::{mocking::MockBlock, GenesisConfig};
use pallet_sudo;
use sp_runtime::{
	traits::{ConstU32, ConstU64},
	BuildStorage,
};

pub type Balance = u128;
pub type AccountId = u64;

pub const ADMIN: AccountId = 1;
pub const USER2: AccountId = 2;
pub const USER3: AccountId = 3;
pub const USER4: AccountId = 4;

// Configure a mock runtime to test the pallet.
#[frame_support::runtime]
mod test_runtime {
	use super::AccountId;

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
	pub type Sudo = pallet_sudo;
	#[runtime::pallet_index(2)]
	pub type PaymentModule = pallet_payment;
	#[runtime::pallet_index(3)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(4)]
	pub type EdgeConnectModule = pallet_edge_connect;
	#[runtime::pallet_index(5)]
	pub type Timestamp = pallet_timestamp;
	#[runtime::pallet_index(6)]
	pub type Assets = pallet_assets;
}

// Parameters and implementation for frame_system::Config for the Test runtime
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = MockBlock<Test>;
	type AccountId = AccountId;
	type AccountData = pallet_balances::AccountData<Balance>;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Nonce = u64;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
}

parameter_types! {
		pub const AssetDeposit: Balance = 100;
		pub const AssetAccountDeposit: Balance = 10;
		pub const ApprovalDeposit: Balance = 1;
		pub const AssetsStringLimit: u32 = 50;
		pub const MetadataDepositBase: Balance = 10;
		pub const MetadataDepositPerByte: Balance = 1;
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

impl pallet_asset_adapter::Config for Test {}

// Implementation of the Payment pallet's configuration for the Test runtime
impl pallet_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TreasuryAccount = TreasuryAccount;
	type WeightInfo = ();
	type MaxKycHashLength = ConstU32<64>;
	type MaxPaymentIdLength = MaxPaymentIdLength;
	type MaxUserIdLength = MaxUserIdLength;
}

parameter_types! {
	pub const TreasuryAccount: AccountId = 999;
}

impl pallet_edge_connect::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 10;
	pub const MaxPaymentIdLength: u32 = 128;
	pub const MaxUserIdLength: u32 = 128;
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

impl pallet_sudo::Config for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<0>;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_sudo::GenesisConfig::<Test> { key: Some(ADMIN) }
		.assimilate_storage(&mut storage)
		.unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10_000), (2, 50_000), (3, 50_000)],
		..Default::default()
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}
