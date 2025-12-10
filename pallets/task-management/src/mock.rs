pub use crate as pallet_task_management;
use cyborg_primitives::constants::{DAYS, HOURS};
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU32, EnsureOriginWithArg},
	weights::constants::RocksDbWeight,
	PalletId,
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureRootWithSuccess, GenesisConfig};
use pallet_edge_connect;
use pallet_payment;
use sp_runtime::{traits::ConstU64, BuildStorage};

pub type Balance = u128;
pub type AccountId = u64;
pub type AssetId = u32;

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
	pub type Timestamp = pallet_timestamp;

	#[runtime::pallet_index(2)]
	pub type EdgeConnectModule = pallet_edge_connect;

	#[runtime::pallet_index(3)]
	pub type TaskManagementModule = pallet_task_management;

	#[runtime::pallet_index(4)]
	pub type PaymentModule = pallet_payment;

	#[runtime::pallet_index(5)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(6)]
	pub type Assets = pallet_assets;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = MockBlock<Test>;
	type AccountId = AccountId;
	type Nonce = u64;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
	type AccountData = pallet_balances::AccountData<u128>;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
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

impl pallet_task_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type TaskConfirmationTimeout = TaskConfirmationTimeout;
}

parameter_types! {
		pub const TaskConfirmationTimeout: u64 = 75;
}

impl pallet_edge_connect::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
		// pub const MaxPaymentIdLength: u32 = 128;
		pub const ExistentialDeposit: u128 = 10;
		pub const MaxUserIdLength: u32 = 128;
		pub const OnDemandRate: Balance = 6;
		pub const SubscriptionRate: Balance = 10;
		pub const PaymentPalletId: PalletId = PalletId(*b"py/paymt");
}

impl pallet_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	// type WeightInfo = ();
	type PalletId = PaymentPalletId;
	type SubscriptionPeriod = ConstU64<{ 30 * DAYS }>;
	type OnDemandPeriod = ConstU64<{ HOURS }>;
	type GracePeriod = ConstU64<{ 4 * DAYS }>;
	type OnDemandRate = OnDemandRate;
	type SubscriptionRate = SubscriptionRate;
	// type TreasuryAccount = TreasuryAccount;
	// type WeightInfo = ();
	type MaxKycHashLength = ConstU32<64>;
	// type MaxPaymentIdLength = MaxPaymentIdLength;
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
		pub const TreasuryAccount: AccountId = 999; // Use AccountId type here
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<0>;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
