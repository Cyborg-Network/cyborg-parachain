pub use crate as pallet_payment;
use frame_support::{derive_impl, parameter_types, weights::constants::RocksDbWeight};
use frame_system::{mocking::MockBlock, GenesisConfig};
use pallet_sudo;
use sp_runtime::{
	traits::{ConstU32, ConstU64},
	BuildStorage,
};

pub type Balance = u128;
pub type AccountId = u128;

pub const ADMIN: AccountId = 1;
pub const USER2: AccountId = 2;
pub const USER3: AccountId = 3;

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
	pub type Sudo = pallet_sudo;
	#[runtime::pallet_index(2)]
	pub type PaymentModule = pallet_payment;
	#[runtime::pallet_index(3)]
	pub type Balances = pallet_balances;
}

// Parameters and implementation for frame_system::Config for the Test runtime
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type AccountData = pallet_balances::AccountData<Balance>;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Nonce = u64;
	type Block = MockBlock<Test>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
}

// Implementation of the Payment pallet's configuration for the Test runtime
impl pallet_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
}

// Parameter types for the Balances pallet (defines token properties such as existential deposit)
parameter_types! {
	pub const ExistentialDeposit: u128=10;
}

// Implementation of the Balances pallet's configuration for the Test runtime
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
}

impl pallet_sudo::Config for Test {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_sudo::GenesisConfig::<Test> { key: Some(ADMIN) }
		.assimilate_storage(&mut storage)
		.unwrap();

	// Initialize the Balances pallet with some default values
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 10_000), // Account 1 starts with 10,000 tokens
			(2, 50_000), // Account 2 (Admin) starts with 50,000 tokens
			(3, 50_000),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}
