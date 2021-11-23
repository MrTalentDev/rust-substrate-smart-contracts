use api_metadata::api::{DefaultConfig, RuntimeApi};
use ink_env::call::{ExecutionInput, Selector};
use parity_scale_codec::Encode;
use rand::Rng;
use sp_keyring::AccountKeyring;
use subxt::{
    extrinsic::PairSigner,
    sp_runtime::traits::{BlakeTwo256, Hash},
    ClientBuilder, Error,
};

#[tokio::test]
async fn instantiate_flipper_constract() -> Result<(), Box<dyn std::error::Error>> {
    let api = ClientBuilder::new()
        .set_url("ws://substrate-contracts-node:9944")
        .build()
        .await?
        .to_runtime_api::<RuntimeApi<DefaultConfig>>();

    let code = std::fs::read("../contract/target/ink/flipper.wasm")?;
    let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let mut constructor_selector: [u8; 4] = Default::default();
    constructor_selector.copy_from_slice(&BlakeTwo256::hash(b"default")[0..4]);

    let constructor_selector = ExecutionInput::new(Selector::new(constructor_selector));
    let salt: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>();
    let result = api
        .tx()
        .contracts()
        .instantiate_with_code(
            1_111_000_000_000_000,
            200_000_000_000,
            code,
            constructor_selector.encode(),
            salt.encode(),
        )
        .sign_and_submit_then_watch(&signer)
        .await?;

    let instantiated = result
        .find_event::<api_metadata::api::contracts::events::Instantiated>()?
        .ok_or_else(|| Error::Other("Failed to find a Instantiated event".into()))?;
    result
        .find_event::<api_metadata::api::system::events::ExtrinsicSuccess>()?
        .ok_or_else(|| Error::Other("Failed to find a ExtrinsicSuccess event".into()))?;

    println!("Contract address: {}", instantiated.contract);

    Ok(())
}
