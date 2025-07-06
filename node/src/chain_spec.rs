use cumulus_primitives_core::ParaId;
use cyborg_runtime as runtime;
use runtime::{AccountId,AuraId, Signature, EXISTENTIAL_DEPOSIT};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use sp_core::crypto::ByteArray;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use hex_literal::hex;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	#[serde(alias = "relayChain", alias = "RelayChain")]
	pub relay_chain: String,
	/// The id of the Parachain.
	#[serde(alias = "paraId", alias = "ParaId")]
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> runtime::SessionKeys {
	runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "BORG".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::builder(
		runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: 1000,
		},
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		1000.into(),
	))
	.with_properties(properties)
	.build()
}

pub fn local_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	// Sudo Account
	let root_key:AccountId = array_bytes::hex_n_into_unchecked("885e1f8a0b2f3a1526d294f9030b9b9f7329cd2657a81f13d9eb1391dfd20415");

	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "BORG".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	let collator_1: AccountId = array_bytes::hex_n_into_unchecked("4c8d547bf124b643d1f35157b90f03a7ceecf928e83a82bb1e88cef0e8449a5b");
	let aura_id1: AuraId = AuraId::from_slice(&hex!("4c8d547bf124b643d1f35157b90f03a7ceecf928e83a82bb1e88cef0e8449a5b")).unwrap();


	let collator_2: AccountId = array_bytes::hex_n_into_unchecked("58de0fd4137c2931ac8e2bb1d227380e8d8d8444d632c71431706d4540744766");
	let aura_id2: AuraId = AuraId::from_slice(&hex!("58de0fd4137c2931ac8e2bb1d227380e8d8d8444d632c71431706d4540744766")).unwrap();

	
	#[allow(deprecated)]
	ChainSpec::builder(
		runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		Extensions {
			relay_chain: "rococo-local".into(),
			// You MUST set this to the correct network!
			para_id: 1000,
		},
	)
	.with_name("Local Testnet")
	.with_id("local")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		vec![
			(
				collator_1,
				aura_id1,
			),
			(
				collator_2,
				aura_id2,
			),
		],
		root_key.clone(),
		1000.into(),
	))
	.with_protocol_id("template-local")
	.with_properties(properties)
	.build()
}


fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	let early_contributors:AccountId = array_bytes::hex_n_into_unchecked("b8afa2f67521bd80e4febceea9dd44a249744596b658b06e943ae265bf5be252");
	let public_sale:AccountId = array_bytes::hex_n_into_unchecked("bef744b4a41a91f56bf8ca2f5dfd92e3d55f2419a620e10bbc967a703708eb5e");
	let miners:AccountId = array_bytes::hex_n_into_unchecked("688d17178101de764a96e0da7fa9f3f3edbf14baa31c1a7d5e4bf6190562742e");
	let foundation:AccountId = array_bytes::hex_n_into_unchecked("4e18ed26e8c176d2e03e86e0b3c384ec7c2ea97414b6e6f20160d74a3819ce7c");
	let founding_team:AccountId = array_bytes::hex_n_into_unchecked("629f61aa89835bea084ad5cc02c65fe5043d6adc4dc214837dc941f6599a5e34");
	let marketing:AccountId = array_bytes::hex_n_into_unchecked("da0f205fba369ea8d1a3dc925aaba2cc7fe691e44e351854ead006b2a926545b");
	let early_backers:AccountId = array_bytes::hex_n_into_unchecked("38a69de6d0e09071458f3f0c8b0298b463aaae1b227be41faa01ecbde6fd1861");
	let presale:AccountId = array_bytes::hex_n_into_unchecked("640c2f92a2744b9ca2fc56b59b9446077c46b4a563df3206596553f735ce5f0c");
	let collator_accounts: Vec<AccountId> = invulnerables.iter().map(|(acc, _)| acc.clone()).collect();
	let candidacy_bond = EXISTENTIAL_DEPOSIT * 16;
	let total_supply: u128 = 20_000_000 * 10u128.pow(12);
	let balances = vec![
		(early_contributors.clone(), total_supply * 2 / 100),
		(public_sale.clone(), total_supply * 25 / 100),
		(miners.clone(), total_supply * 20 / 100),
		(foundation.clone(), total_supply * 10 / 100),
		(founding_team.clone(), total_supply * 10 / 100),
		(marketing.clone(), total_supply * 10 / 100),
		(early_backers.clone(), total_supply * 20 / 100),
		(presale.clone(), total_supply * 3 / 100),
		(collator_accounts[0].clone(), candidacy_bond * 2 ),
		(collator_accounts[1].clone(), candidacy_bond * 2 ),

	];

	serde_json::json!({
		"balances": {
				"balances": balances,
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16, // 0.001 BORG * 16
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) }
	})
}