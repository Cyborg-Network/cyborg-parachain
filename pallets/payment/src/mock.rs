use frame_support::{
	derive_impl, parameter_types, traits::VariantCountOf, weights::constants::RocksDbWeight,
};
use frame_system::{mocking::MockBlock, GenesisConfig};
use sp_runtime::{
	traits::{ConstU32, ConstU64},
	BuildStorage,
};

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
	pub type PaymentModule = crate;
	#[runtime::pallet_index(2)]
	pub type Balances = pallet_balances;
}

// Parameters and implementation for frame_system::Config for the Test runtime
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Nonce = u64;
	type Block = MockBlock<Test>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
	type AccountData = pallet_balances::AccountData<Balance>;
}

// Implementation of the Payment pallet's configuration for the Test runtime
impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
}

// Parameter types for the Balances pallet (defines token properties such as existential deposit)
parameter_types! {
	pub const ExistentialDeposit: u128=500;
}

// Implementation of the Balances pallet's configuration for the Test runtime
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = GenesisConfig::<Test>::default().build_storage().unwrap();

	// Initialize the Balances pallet with some default values
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 10_000), // Account 1 starts with 10,000 tokens
			(2, 50_000), // Account 2 (Admin) starts with 50,000 tokens
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}
