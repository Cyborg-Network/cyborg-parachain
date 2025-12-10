pub use crate as pallet_task_management;
use cyborg_primitives::constants::{DAYS, HOURS};
use cyborg_primitives::payment::{PaymentMode, PaymentRates};
use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU32, EnsureOriginWithArg},
    weights::constants::RocksDbWeight,
    PalletId,
};
use orml_traits::{BalanceStatus, GetByKey, MultiCurrency, MultiReservableCurrency};
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
    pub type Tokens = orml_tokens;
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

// Mock existential deposits for ORML tokens
parameter_types! {
    pub const NativeExistentialDeposit: Balance = 10;
    pub const UsdcExistentialDeposit: Balance = 1_000;
    pub const DefaultExistentialDeposit: Balance = 1;
}

pub struct MockExistentialDeposits;

impl GetByKey<AssetId, Balance> for MockExistentialDeposits {
    fn get(k: &AssetId) -> Balance {
        match k {
            0 => NativeExistentialDeposit::get(),
            1 => UsdcExistentialDeposit::get(),
            _ => DefaultExistentialDeposit::get(),
        }
    }
}

// ORML tokens configuration
parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl orml_tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = i128;
    type CurrencyId = AssetId;
    type WeightInfo = ();
    type ExistentialDeposits = MockExistentialDeposits;
    type CurrencyHooks = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type DustRemovalWhitelist = ();
}

// Payment rates implementation for mock
pub struct MockPaymentRates;

impl PaymentRates<AssetId, Balance> for MockPaymentRates {
    fn get_rate(asset_id: AssetId, mode: PaymentMode) -> Balance {
        match asset_id {
            0 => match mode {  // Native token
                PaymentMode::OnDemand => 10,
                PaymentMode::Subscription => 100,
            },
            1 => match mode {  // USDC
                PaymentMode::OnDemand => 2_000_000,
                PaymentMode::Subscription => 10_000_000,
            },
            _ => match mode {  // Default for other assets
                PaymentMode::OnDemand => 2,
                PaymentMode::Subscription => 10,
            },
        }
    }
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
    pub const ExistentialDeposit: u128 = 10;
    pub const MaxUserIdLength: u32 = 128;
    pub const PaymentPalletId: PalletId = PalletId(*b"py/paymt");
    pub const SubscriptionPeriod: u64 = 30 * DAYS;
    pub const OnDemandPeriod: u64 = HOURS;
    pub const GracePeriod: u64 = 4 * DAYS;
}

impl pallet_payment::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = PaymentPalletId;
    type SubscriptionPeriod = ConstU64<{ 30 * DAYS }>;
    type OnDemandPeriod = ConstU64<{ HOURS }>;
    type GracePeriod = ConstU64<{ 4 * DAYS }>;
    type Rate = MockPaymentRates;
    type Asset = orml_tokens::Pallet<Test>; // Use ORML tokens
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
    pub const TreasuryAccount: AccountId = 999;
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
