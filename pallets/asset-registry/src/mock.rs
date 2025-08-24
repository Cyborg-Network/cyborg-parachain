pub use crate as pallet_asset_registry;
use frame_support::{
	derive_impl, parameter_types, traits::ConstU32, weights::constants::RocksDbWeight,
};
use frame_system::mocking::MockBlock;
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage};
use xcm::latest::prelude::*;

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
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(2)]
	pub type AssetRegistry = pallet_asset_registry;
}

parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = MockBlock<Test>;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Nonce = u64;
	type Hash = H256;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type SS58Prefix = SS58Prefix;
	type AccountData = pallet_balances::AccountData<u128>;
}

parameter_types! {
		pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

#[derive(Clone)]
pub struct MockWeightless(pub xcm::latest::Xcm<RuntimeCall>);

// Implement PreparedMessage trait for MockWeightless
impl xcm::v5::PreparedMessage for MockWeightless {
	fn weight_of(&self) -> xcm::latest::Weight {
		// Return a minimal weight for testing
		xcm::latest::Weight::from_parts(1_000_000, 0)
	}
}

// Mock XCM executor - Use Weightless as Prepared type
pub struct MockXcmExecutor;

impl ExecuteXcm<RuntimeCall> for MockXcmExecutor {
	type Prepared = MockWeightless;

	fn prepare(
		message: xcm::latest::Xcm<RuntimeCall>,
	) -> Result<Self::Prepared, xcm::latest::Xcm<RuntimeCall>> {
		Ok(MockWeightless(message))
	}

	fn execute(
		_origin: impl Into<xcm::latest::Location>,
		pre: Self::Prepared,
		_id: &mut xcm::latest::XcmHash,
		_weight_credit: xcm::latest::Weight,
	) -> xcm::latest::Outcome {
		// For testing purposes, we'll just return success
		// You can add logic here to inspect the message if needed
		let _message = pre.0; // Extract the original message
		xcm::latest::Outcome::Complete {
			used: xcm::latest::Weight::from_parts(1_000_000, 0),
		}
	}

	fn charge_fees(
		_location: impl Into<xcm::latest::Location>,
		_fees: xcm::latest::Assets,
	) -> Result<(), xcm::latest::Error> {
		Ok(())
	}
}

impl pallet_asset_registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = MockXcmExecutor;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	// Initialize the system to block 1 so events work
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 1000000000000), (2, 1000000000000)],
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
