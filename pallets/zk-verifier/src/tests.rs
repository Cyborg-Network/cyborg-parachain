// MIT License

// Copyright (c) 2022 Bright Inventions

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// MODIFICATIONS:
// - Modified by ZCalz on 2024-10-22: Updated extrinsic parameter calls

#![cfg(test)]

use crate::{mock::*, *};

use frame_support::{assert_err, assert_ok};

const ALICE_ACCOUNT_ID: u64 = 2;
const BOB_ACCOUNT_ID: u64 = 3;
const taskId: TaskId = 0;

#[test]
fn test_setup_verification() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		// Call the function and verify it works correctly
		assert_ok!(ZKVerifierModule::setup_verification(
			RuntimeOrigin::none(),
			taskId,
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));

		// Updated storage check
		let stored_vk = VerificationKeyStorage::<Test>::get(taskId);
		assert_eq!(stored_vk.is_some(), true);

		// Verify the event was emitted
		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}
#[test]
fn test_not_supported_vk_curve() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bn128", None);
		assert_err!(
			ZKVerifierModule::setup_verification(
				RuntimeOrigin::none(),
				taskId,
				prepare_correct_public_inputs_json().as_bytes().into(),
				vk.as_bytes().into()
			),
			Error::<Test>::NotSupportedCurve
		);
		let events = zk_events();
		assert_eq!(events.len(), 0);
	});
}

#[test]
fn test_not_supported_vk_protocol() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("-", "bls12381", None);
		assert_err!(
			ZKVerifierModule::setup_verification(
				RuntimeOrigin::none(),
				taskId,
				prepare_correct_public_inputs_json().as_bytes().into(),
				vk.as_bytes().into()
			),
			Error::<Test>::NotSupportedProtocol
		);
		let events = zk_events();
		assert_eq!(events.len(), 0);
	});
}

#[test]
fn test_too_long_verification_key() {
	new_test_ext().execute_with(|| {
		let too_long_vk = vec![0; (<Test as Config>::MaxVerificationKeyLength::get() + 1) as usize];
		assert_err!(
			ZKVerifierModule::setup_verification(
				RuntimeOrigin::none(),
				taskId,
				prepare_correct_public_inputs_json().as_bytes().into(),
				too_long_vk
			),
			Error::<Test>::TooLongVerificationKey
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_too_long_public_inputs() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::setup_verification(
				RuntimeOrigin::none(),
				taskId,
				vec![0; (<Test as Config>::MaxPublicInputsLength::get() + 1) as usize],
				prepare_vk_json("groth16", "bls12381", None)
					.as_bytes()
					.into()
			),
			Error::<Test>::TooLongPublicInputs
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_public_inputs_mismatch() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::setup_verification(
				RuntimeOrigin::none(),
				taskId,
				prepare_empty_public_inputs_json().as_bytes().into(),
				prepare_vk_json("groth16", "bls12381", None)
					.as_bytes()
					.into()
			),
			Error::<Test>::PublicInputsMismatch
		);

		// Check storage to ensure no key was set
		assert!(!VerificationKeyStorage::<Test>::contains_key(taskId));
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_too_long_proof() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::verify(
				RuntimeOrigin::none(),
				taskId,
				vec![0; (<Test as Config>::MaxProofLength::get() + 1) as usize]
			),
			Error::<Test>::TooLongProof
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_not_supported_proof_protocol() {
	let proof = prepare_proof_json("-", "bls12381", None);

	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::verify(RuntimeOrigin::none(), taskId, proof.as_bytes().into()),
			Error::<Test>::NotSupportedProtocol
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_not_supported_proof_curve() {
	let proof = prepare_proof_json("groth16", "bn128", None);

	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::verify(RuntimeOrigin::none(), taskId, proof.as_bytes().into()),
			Error::<Test>::NotSupportedCurve
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_empty_proof() {
	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::verify(RuntimeOrigin::none(), taskId, Vec::new()),
			Error::<Test>::ProofIsEmpty
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_verify_without_verification_key() {
	let proof = prepare_proof_json("groth16", "bls12381", None);

	new_test_ext().execute_with(|| {
		assert_err!(
			ZKVerifierModule::verify(RuntimeOrigin::none(), taskId, proof.as_bytes().into()),
			Error::<Test>::VerificationKeyIsNotSet
		);
		assert_eq!(zk_events().len(), 0);
	});
}

#[test]
fn test_verification_success() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		let proof = prepare_proof_json("groth16", "bls12381", None);

		assert_ok!(ZKVerifierModule::setup_verification(
			RuntimeOrigin::none(),
			taskId,
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_ok!(ZKVerifierModule::verify(
			RuntimeOrigin::signed(ALICE_ACCOUNT_ID),
			taskId,
			proof.as_bytes().into()
		));

		let events = zk_events();
		assert_eq!(events.len(), 3);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
		assert_eq!(events[1], Event::<Test>::VerificationProofSet);
		assert_eq!(
			events[2],
			Event::<Test>::VerificationSuccess {
				who: ALICE_ACCOUNT_ID
			}
		);
	});
}

#[test]
fn test_verification_failed() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		let proof = prepare_proof_json("groth16", "bls12381", None);

		assert_ok!(ZKVerifierModule::setup_verification(
			RuntimeOrigin::none(),
			taskId,
			prepare_incorrect_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_ok!(ZKVerifierModule::verify(
			RuntimeOrigin::signed(BOB_ACCOUNT_ID),
			taskId,
			proof.as_bytes().into()
		));

		let events = zk_events();
		assert_eq!(events.len(), 3);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
		assert_eq!(events[1], Event::<Test>::VerificationProofSet);
		assert_eq!(events[2], Event::<Test>::VerificationFailed);
	});
}

#[test]
fn test_could_not_create_proof() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		let proof = prepare_proof_json("groth16", "bls12381", Some("12".to_owned()));

		assert_ok!(ZKVerifierModule::setup_verification(
			RuntimeOrigin::none(),
			taskId,
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_err!(
			ZKVerifierModule::verify(RuntimeOrigin::none(), taskId, proof.as_bytes().into()),
			Error::<Test>::ProofCreationError
		);

		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}

#[test]
fn test_could_not_create_verification_key() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", Some("12".to_owned()));
		let proof = prepare_proof_json("groth16", "bls12381", None);

		assert_ok!(ZKVerifierModule::setup_verification(
			RuntimeOrigin::none(),
			taskId,
			prepare_correct_public_inputs_json().as_bytes().into(),
			vk.as_bytes().into()
		));
		assert_err!(
			ZKVerifierModule::verify(RuntimeOrigin::none(), taskId, proof.as_bytes().into()),
			Error::<Test>::VerificationKeyCreationError
		);

		let events = zk_events();
		assert_eq!(events.len(), 1);
		assert_eq!(events[0], Event::<Test>::VerificationSetupCompleted);
	});
}

fn prepare_correct_public_inputs_json() -> String {
	r#"[
 "12"
]"#
		.to_owned()
}

fn prepare_incorrect_public_inputs_json() -> String {
	r#"[
 "3"
]"#
		.to_owned()
}

fn prepare_empty_public_inputs_json() -> String {
	r#"[
]"#
		.to_owned()
}

fn prepare_vk_json(protocol: &str, curve: &str, alpha_x: Option<String>) -> String {
	let alpha_x = alpha_x.unwrap_or_else(|| "2417420058161902631695569321985275527817337553240735969068630412919230058600548397578577183742111992841943587142680".to_owned());
	let vk_template = r#"{
"protocol": "<protocol>",
"curve": "<curve>",
"nPublic": 1,
"vk_alpha_1": [
   "<alpha_x>",
   "2683193963041639430431668252069589353703764749562535314981925385889474793061455502785968498855669710056680025802535",
   "1"
],
"vk_beta_2": [
   [
   "2953983861911780746898420772852203750596202163211813473761616529894571940032171065334774419373056700627707738200018",
   "3062465588861097636655055190501059315624734570742089309263797407021640154269222765149244340402777629537231482465213"
   ],
   [
   "2880510548434910442614869111285946610418075557776097505115113030863387119802265689270335925248001883102867749676243",
   "2872114062532568575643729173452461066994643453813848213872870173636132169046691827766994227240293333106164659529444"
   ],
   [
   "1",
   "0"
   ]
],
"vk_gamma_2": [
    [
    "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160",
    "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758"
    ],
    [
    "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905",
    "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582"
    ],
    [
    "1",
    "0"
    ]
],
"vk_delta_2": [
    [
    "1397400294785329269149248027941029918234275798984995986592789994215372037046682288247459925132482655775231958770596",
    "3613651892030917982825314322568444757238870140073427833524931882395488683192849483836696311878674061447378155414322"
    ],
    [
    "1454420022135097547429203607513890428221900276713697693498600894391966225725356692084173923746366083520797626734711",
    "2405306655262521121779739123612338596090750073099847349336699337941746231436397110773618181083856700942862129820841"
    ],
    [
    "1",
    "0"
    ]
],
"vk_alphabeta_12": [
    [
    [
    "437761028892224030465034124633217268719956342872559616008634729284766695298988063647325250812018792056937766615471",
    "2999394135376152607622366876448622373445582872299531408525147517624498217644608450506056086700936703928514704064261"
    ],
    [
    "3790463607805336527949393722330738149514940802753341300767573186059738586132357982589584572188455144653483526564746",
    "921836187515472488254938786547195013453267292824125099383072882353289064169744390896404473125029705474288494798314"
    ],
    [
    "1508236233377234567827114620858145363511004510596666326403249659168776157798943222844325580955232907567566667625003",
    "3575535003105597889724716301018784934429688500083929741894285412422510995949379530614805831071483291549658546954371"
    ]
    ],
    [
    [
    "3785220960014277412034694592863705652133531504406398705042993478660513767217489443009741763190154176262309373216597",
    "449323421898093201829817731810072028407746890084253250422696110923119766623806607498486827118410872300484828826217"
    ],
    [
    "1699227721327300072207096717806024682229688959499287462332798396856999811919927621303990692220790062830828210894595",
    "3256071168762144542887631756797467730909954071672365830591432096675734476136634634750995051556249166553938833471640"
    ],
    [
    "3981041298268096327563938150273814160101198859016900206793418664618794328703842591186375302820586562545170299574243",
    "3899273791593867671292235821132306394536495869179149599490110076955624892882281866127672380389653164615423489095918"
    ]
    ]
],
"IC": [
    [
    "1036455169342233390855996586834520647962171510914420928779905953251272176363349160512017514969413843826714495861777",
    "3225757548975669202743314017707154170140342810479555354528303455797434256089415962868447574306245203533729979725838",
    "1"
    ],
    [
    "2306767568146465899824632338747274961711075325739057886746993285987967410538122442295923393427774655394152050218360",
    "1110686736735022843500989850943596336256510944040379817126812118843722981304262779720098389756327870602977197635083",
    "1"
    ]
]
}"#;
	vk_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<alpha_x>", &alpha_x)
}

fn prepare_proof_json(protocol: &str, curve: &str, pi_a_x: Option<String>) -> String {
	let pi_a_x = pi_a_x.unwrap_or_else(|| "1547868284561670884744470829066291861753711715427536197016979117727657722537367306855408779073400007356480755992286".to_owned());
	let proof_template = r#"{
"pi_a": [
"<pi_a_x>",
    "133377702143528739575377729631360601614088262416333931136172973337607317017542609318946667454426700160620492918070",
    "1"
],
"pi_b": [
    [
    "3464179927623990666132434581669710292812271436336621246126774308069940684644800766694467705159555008883836001203558",
    "2546213637341159614042232103352468058136925633034122854640067781563520449770334670597953179425897845578304324932654"
    ],
    [
    "1727172519477219519750367293438016239792036515829871417520013243406611034907195588907593103368826194109213319586533",
    "1608709552654556864133663038831358765687167633553533833302139692670076873672935498325809703404354703063813928303923"
    ],
    [
    "1",
    "0"
    ]
],
"pi_c": [
    "1754096103716358561952826128249523421393931227029702817784288419733418512708632119712049074095306383315056978720954",
    "2834250288052560472935431224341595955480629006732618887386362957441961005785403404522081920080207211610068590548972",
    "1"
],
"protocol": "<protocol>",
"curve": "<curve>"
}"#;

	proof_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<pi_a_x>", &pi_a_x)
}
