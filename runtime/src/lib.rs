#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub mod configs;
pub mod weights;

mod oracle_router;
use oracle_router::OracleRouter;

extern crate alloc;
use smallvec::smallvec;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU8},
	weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

use weights::ExtrinsicBaseWeight;

pub use frame_system::EnsureRoot;

pub use cyborg_primitives::{
	oracle::{DummyCombineData, OracleKey, OracleValue, OracleWorkerFormat, ProcessStatus},
	task::TaskId,
	worker::{WorkerId, WorkerType},
};

pub use pallet_edge_connect;
pub use pallet_neuro_zk;
pub use pallet_payment;
pub use pallet_status_aggregator;
pub use pallet_task_management;
pub use pallet_zk_verifier;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
#[docify::export(template_signed_extra)]
pub type SignedExtra = cumulus_pallet_weight_reclaim::StorageWeightReclaim<
	Runtime,
	(
		frame_system::CheckNonZeroSender<Runtime>,
		frame_system::CheckSpecVersion<Runtime>,
		frame_system::CheckTxVersion<Runtime>,
		frame_system::CheckGenesis<Runtime>,
		frame_system::CheckEra<Runtime>,
		frame_system::CheckNonce<Runtime>,
		frame_system::CheckWeight<Runtime>,
		pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
		frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
	),
>;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = MILLIUNIT / 10;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

parameter_types! {
	pub RootOperatorAccountId: AccountId = AccountId::from([0xffu8; 32]);
}

#[cfg(feature = "runtime-benchmarks")]
use core::marker::PhantomData;
//use std::simd::SupportedLaneCount;
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::account;
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::{traits::Get, BoundedVec};

// Struct to implement the BenchmarkHelper trait for benchmarking. It takes a generic MaxFeedValues,
// which defines the maximum number of values (or pairs) the benchmark will generate.
// The `PhantomData<MaxFeedValues>` is used because MaxFeedValues is a type-level constant,
// and we need to associate it with this struct but don't actually store it.
#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkHelperImpl<MaxFeedValues>(PhantomData<MaxFeedValues>);

// Implementing the orml_oracle::BenchmarkHelper trait for the BenchmarkHelperImpl struct.
// The trait is specialized for key-value pairs with the key being a tuple of (AccountId, WorkerId)
// and the value being ProcessStatus. MaxFeedValues is a type constant that defines the upper limit
// on the number of pairs that can be generated.
#[cfg(feature = "runtime-benchmarks")]
impl<MaxFeedValues> orml_oracle::BenchmarkHelper<OracleKey<AccountId>, OracleValue, MaxFeedValues>
	for BenchmarkHelperImpl<MaxFeedValues>
where
	MaxFeedValues: Get<u32>,
{
	// The required method from the BenchmarkHelper trait, which we are customizing for benchmarking the status-aggregator pallet.
	// This method outputs key-value pairs where the key is a tuple of (AccountId, WorkerId) and the value is ProcessStatus.
	fn get_currency_id_value_pairs() -> BoundedVec<(OracleKey<AccountId>, OracleValue), MaxFeedValues>
	{
		let mut pairs: BoundedVec<(OracleKey<AccountId>, OracleValue), MaxFeedValues> =
			BoundedVec::default();

		let max = MaxFeedValues::get().min(100);

		for seed in 0..max {
			let account: AccountId = account("oracle", 0, seed);
			let worker_id: WorkerId = seed as u64;

			// Alternate between Miner and Zk entries
			if seed % 2 == 0 {
				// MinerStatus entry
				let key = OracleKey::Miner(OracleWorkerFormat {
					id: (account.clone(), worker_id),
					worker_type: WorkerType::Executable,
				});

				let value = OracleValue::MinerStatus(ProcessStatus {
					online: seed % 2 == 0,
					available: seed % 3 == 0,
				});

				pairs
					.try_push((key, value))
					.expect("Exceeded MaxFeedValues");
			} else {
				// ZkProofResult entry
				let task_id: TaskId = seed as u64;

				let key = OracleKey::NzkProofResult(task_id);
				let value = OracleValue::ZkProofResult(seed % 3 == 0);

				pairs
					.try_push((key, value))
					.expect("Exceeded MaxFeedValues");
			}
		}

		pairs
	}
}

/*
#[cfg(feature = "runtime-benchmarks")]
impl<MaxFeedValues> BenchmarkHelperImpl<MaxFeedValues>
where
	MaxFeedValues: Get<u32>,
{
	// This method generates key-value pairs of (AccountId, WorkerId) -> ProcessStatus,
	// used in the context of benchmarking the status-aggregator pallet.
	// The number of pairs generated is limited by the value of MaxFeedValues.

	fn status_aggregator_benchmark_data(
	) -> BoundedVec<((AccountId, WorkerId), ProcessStatus), MaxFeedValues> {
		// Initialize a BoundedVec to store the generated key-value pairs,
		// constrained by MaxFeedValues, which defines the upper limit.
		let mut pairs: BoundedVec<((AccountId, WorkerId), ProcessStatus), MaxFeedValues> =
			BoundedVec::default();

		let max_limit = 100;

		// Loop to generate pseudo-random key-value pairs for benchmarking,
		// using max_values to control the number of iterations.
		for seed in 0..max_limit {
			//generate pseudo-random accountId
			let account_id: AccountId = account("benchmark_account", 0, seed);

			//generate pseudo-random workerId
			let worker_id: WorkerId = (seed as u64) * 12345;

			//generate pseudo-random ProcessStatus
			let process_status = ProcessStatus {
				online: seed % 2 == 0,
				available: seed % 3 == 0,
			};

			// Add the generated key-value pair to the BoundedVec.
			// The number of entries pushed is limited by MaxFeedValues.
			pairs
				.try_push(((account_id, worker_id), process_status))
				.expect("Exceeded MaxFeedValues limit");
		}

		pairs
	}
}
*/

impl orml_oracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnNewData = OracleRouter;
	type CombineData = DummyCombineData<Runtime>;
	type Time = Timestamp;
	type OracleKey = OracleKey<Self::AccountId>;
	type OracleValue = OracleValue;
	type RootOperatorAccountId = RootOperatorAccountId;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Members = OracleMembership;
	#[cfg(feature = "runtime-benchmarks")]
	type Members = OracleMembershipWrapper;
	type MaxHasDispatchedSize = ConstU32<8>;
	type WeightInfo = weights::orml_oracle::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type MaxFeedValues = ConstU32<500>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxFeedValues = ConstU32<500>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelperImpl<Self::MaxFeedValues>;
}

impl pallet_membership::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRoot<AccountId>;
	type RemoveOrigin = EnsureRoot<AccountId>;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type PrimeOrigin = EnsureRoot<AccountId>;
	type MembershipInitialized = ();
	type MembershipChanged = ();
	type MaxMembers = ConstU32<16>;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

/// OracleMembership wrapper used by benchmarks
#[cfg(feature = "runtime-benchmarks")]
pub struct OracleMembershipWrapper;

#[cfg(feature = "runtime-benchmarks")]
impl frame_support::traits::SortedMembers<AccountId> for OracleMembershipWrapper {
	fn sorted_members() -> Vec<AccountId> {
		OracleMembership::sorted_members()
	}

	fn add(account: &AccountId) {
		frame_support::assert_ok!(OracleMembership::add_member(
			frame_system::RawOrigin::Root.into(),
			account.to_owned().into()
		));
	}
}

impl pallet_edge_connect::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_edge_connect::SubstrateWeight<Runtime>;
}

impl pallet_task_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_task_management::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MaxKycHashLength: u32 = 64;
	pub const MaxPaymentIdLength: u32 = 128;
	pub const MaxUserIdLength: u32 = 128;
}

impl pallet_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = weights::pallet_payment::SubstrateWeight<Runtime>;
	type MaxKycHashLength = MaxKycHashLength;
	type MaxPaymentIdLength = MaxPaymentIdLength;
	type MaxUserIdLength = MaxUserIdLength;
}

parameter_types! {
		pub const MaxBlockRangePeriod: BlockNumber = 50u32; // Set the max block range to 100 blocks
}

impl pallet_status_aggregator::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_status_aggregator::SubstrateWeight<Runtime>;
	type MaxBlockRangePeriod = MaxBlockRangePeriod;
	type ThresholdUptimeStatus = ConstU8<75>;
	type MaxAggregateParamLength = ConstU32<300>;
	type WorkerInfoHandler = EdgeConnect;
}

impl pallet_neuro_zk::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_neuro_zk::SubstrateWeight<Runtime>;
	type AcceptanceThreshold = ConstU8<75>;
	type AggregateLength = ConstU32<1>;
	type NzkTaskInfoHandler = TaskManagement;
}

parameter_types! {
	pub const MaxPublicInputsLength: u32 = 9;
	pub const MaxVerificationKeyLength: u32 = 4143;
	pub const MaxProofLength: u32 = 1133;
}

impl pallet_zk_verifier::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxPublicInputsLength = MaxPublicInputsLength;
	type MaxProofLength = MaxProofLength;
	type MaxVerificationKeyLength = MaxVerificationKeyLength;
	type WeightInfo = ();
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: alloc::borrow::Cow::Borrowed("cyborg-runtime"),
	impl_name: alloc::borrow::Cow::Borrowed("cyborg-runtime"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: apis::RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;
/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

/// Aura consensus hook
type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

// Create the runtime by composing the FRAME pallets that were previously configured.
#[frame_support::runtime]
mod runtime {
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
	pub struct Runtime;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;
	#[runtime::pallet_index(1)]
	pub type ParachainSystem = cumulus_pallet_parachain_system;
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp;
	#[runtime::pallet_index(3)]
	pub type ParachainInfo = parachain_info;

	// Monetary stuff.
	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances;
	#[runtime::pallet_index(11)]
	pub type TransactionPayment = pallet_transaction_payment;

	// Governance
	#[runtime::pallet_index(15)]
	pub type Sudo = pallet_sudo;

	// Collator support. The order of these 4 are important and shall not change.
	#[runtime::pallet_index(20)]
	pub type Authorship = pallet_authorship;
	#[runtime::pallet_index(21)]
	pub type CollatorSelection = pallet_collator_selection;
	#[runtime::pallet_index(22)]
	pub type Session = pallet_session;
	#[runtime::pallet_index(23)]
	pub type Aura = pallet_aura;
	#[runtime::pallet_index(24)]
	pub type AuraExt = cumulus_pallet_aura_ext;

	// XCM helpers.
	#[runtime::pallet_index(30)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue;
	#[runtime::pallet_index(31)]
	pub type PolkadotXcm = pallet_xcm;
	#[runtime::pallet_index(32)]
	pub type CumulusXcm = cumulus_pallet_xcm;
	#[runtime::pallet_index(33)]
	pub type MessageQueue = pallet_message_queue;

	#[runtime::pallet_index(40)]
	pub type Oracle = orml_oracle;

	#[runtime::pallet_index(41)]
	pub type OracleMembership = pallet_membership;

	#[runtime::pallet_index(42)]
	pub type EdgeConnect = pallet_edge_connect;

	#[runtime::pallet_index(43)]
	pub type TaskManagement = pallet_task_management;

	#[runtime::pallet_index(44)]
	pub type StatusAggregator = pallet_status_aggregator;

	#[runtime::pallet_index(45)]
	pub type Payment = pallet_payment;

	#[runtime::pallet_index(46)]
	pub type ZKVerifier = pallet_zk_verifier;

	#[runtime::pallet_index(47)]
	pub type NeuroZk = pallet_neuro_zk;
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
