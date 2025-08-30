use crate::{
	AccountId, AllPalletsWithSystem, Balances, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeOrigin, WeightToFee, XcmpQueue,
};

use alloc::vec;
use alloc::vec::Vec;
use frame_support::{
	parameter_types,
	traits::{ConstU32, Contains, Disabled, Everything, Nothing},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::impls::ToAuthor;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowKnownQueryResponses,
	AllowTopLevelPaidExecutionFrom, Case, DenyReserveTransferToRelayChain, DenyThenTry,
	EnsureXcmOrigin, FixedWeightBounds, FrameTransactionalProcessor, FungibleAdapter, ParentIsPreset,
	RelayChainAsNative, SendXcmFeeToAccount, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	TrailingSetTopicAsId, UsingComponents, WithComputedOrigin, WithUniqueTopic,
	XcmFeeManagerFromComponents,
};
use xcm_executor::traits::MatchesFungible;
use xcm_executor::XcmExecutor;

parameter_types! {
	pub const RelayLocation: Location = Location::parent();
	pub const RelayNetwork: Option<NetworkId> = Some(NetworkId::Polkadot);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation = [
		GlobalConsensus(RelayNetwork::get().unwrap_or(NetworkId::Polkadot)),
		Parachain(ParachainInfo::parachain_id().into())].into();

		pub AssetHubLocation: Location = Location::new(
			1, // parent: Polkadot relay chain
			[Parachain(1000)]  // Asset Hub parachain ID
	);
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub struct RelayOrAssetHubFungible;
impl MatchesFungible<u128> for RelayOrAssetHubFungible {
	fn matches_fungible(a: &Asset) -> Option<u128> {
		match a {
			Asset {
				id: AssetId(relay_location),
				fun: Fungible(amount),
			} if *relay_location == RelayLocation::get() => Some(*amount),
			Asset {
				id: AssetId(asset_hub_location),
				fun: Fungible(amount),
			} if *asset_hub_location == AssetHubLocation::get() => Some(*amount),
			Asset {
				id: AssetId(asset_hub_usdt_location),
				fun: Fungible(amount),
			} if *asset_hub_usdt_location == AssetHubUsdtLocation::get() => Some(*amount),
			Asset {
				id: AssetId(asset_hub_usdc_location),
				fun: Fungible(amount),
			} if *asset_hub_usdc_location == AssetHubUsdcLocation::get() => Some(*amount),
			_ => None,
		}
	}
}

parameter_types! {
		pub TrustedTeleporters: Vec<(Location, Vec<AssetId>)> = vec![
				(
						RelayLocation::get(),
						vec![AssetId(RelayLocation::get())] // DOT
				),
				(
						AssetHubLocation::get(),
						vec![
						AssetId(AssetHubUsdtLocation::get()), // USDT
						AssetId(AssetHubUsdcLocation::get())  // USDC
						]

				)
		];

		pub TrustedReserves: Vec<(Location, Vec<AssetId>)> = vec![
				(
						RelayLocation::get(),
						vec![AssetId(RelayLocation::get())] // DOT
				),
				(
						AssetHubLocation::get(),
						vec![
								AssetId(AssetHubLocation::get()), // Asset Hub native
								AssetId(AssetHubUsdtLocation::get()), // USDT
								AssetId(AssetHubUsdcLocation::get())  // USDC
						]
				)
		];
}

parameter_types! {
		pub AssetHubFeePerSecond: (AssetId, u128) =
				(AssetId(AssetHubLocation::get()), 1_000_000); // 0.000001 DOT per second

				pub AssetHubUsdtFeePerSecond: (AssetId, u128) =
				(AssetId(AssetHubUsdtLocation::get()), 1_000_000); // Fee rate for USDT
}

// teleportation configuration
parameter_types! {
		pub const AssetHubUsdtAssetId: u128 = 1984;
		pub const AssetHubUsdcAssetId: u128 = 1337;
		pub AssetHubUsdtLocation: Location = Location::new(
				1,
				[Parachain(1000), GeneralIndex(AssetHubUsdtAssetId::get())]
		);
		pub AssetHubUsdcLocation: Location = Location::new(
			1,
			[Parachain(1000), GeneralIndex(AssetHubUsdcAssetId::get())]
	);
}

parameter_types! {
		pub RelayDot: (AssetFilter, Location) = (Wild(All), RelayLocation::get());
		pub AssetHubUsdt: (AssetFilter, Location) = (Wild(All), AssetHubUsdtLocation::get());
		pub AssetHubUsdc: (AssetFilter, Location) = (Wild(All), AssetHubUsdcLocation::get());
		pub AssetHubNative: (AssetFilter, Location) = (Wild(All), AssetHubLocation::get());
}

type IsTeleporter = (
	// Allow teleportation of DOT from Relay Chain
	Case<RelayDot>,
	// Allow teleportation of USDT from Asset Hub
	Case<AssetHubUsdt>,
	// Allow teleportation of USDC from Asset Hub
	Case<AssetHubUsdc>,
);

type IsReserve = (
	// Reserve transfers for Relay Chain assets
	Case<RelayDot>,
	// Reserve transfers for Asset Hub assets
	Case<AssetHubNative>,
	// Reserve transfers for Asset Hub USDT
	Case<AssetHubUsdt>,
	// Reserve transfers for Asset Hub USDC
	Case<AssetHubUsdc>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Allow both relay chain assets and Asset Hub assets
	RelayOrAssetHubFungible,
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

parameter_types! {
	pub AssetHubFreeExecution: Location = Location::new(1, [Parachain(1000)]);
}

impl Contains<Location> for AssetHubFreeExecution {
	fn contains(location: &Location) -> bool {
		// Allow execution from Asset Hub parachain
		*location == AssetHubLocation::get()
	}
}

pub struct ParentOrParentsExecutivePlurality;
impl Contains<Location> for ParentOrParentsExecutivePlurality {
	fn contains(location: &Location) -> bool {
		matches!(
			location.unpack(),
			(1, [])
				| (
					1,
					[Plurality {
						id: BodyId::Executive,
						..
					}]
				)
		)
	}
}

pub type Barrier = TrailingSetTopicAsId<
	DenyThenTry<
		DenyReserveTransferToRelayChain,
		(
			TakeWeightCredit,
			WithComputedOrigin<
				(
					AllowTopLevelPaidExecutionFrom<Everything>,
					AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
					AllowExplicitUnpaidExecutionFrom<AssetHubFreeExecution>,
					// ^^^ Parent and its exec plurality get free execution
					// Allow teleportation from trusted locations
					AllowKnownQueryResponses<PolkadotXcm>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
>;

// Define the account to which the fees will be sent
parameter_types! {
	pub TreasuryAccount: AccountId =
				pallet_treasury::Pallet::<Runtime>::account_id();
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type XcmEventEmitter = PolkadotXcm;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = IsReserve;
	type IsTeleporter = IsTeleporter;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ToAuthor<Runtime>>; // Using the fee manager to handle XCM fees
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = XcmFeeManagerFromComponents<
		(),
		SendXcmFeeToAccount<Self::AssetTransactor, TreasuryAccount>, // Convert the weight of XCM message into a fee
	>;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = PolkadotXcm;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	// ^ Disable dispatchable execute on the XCM pallet.
	// Needs to be `Everything` for local testing.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AuthorizedAliasConsideration = Disabled;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
