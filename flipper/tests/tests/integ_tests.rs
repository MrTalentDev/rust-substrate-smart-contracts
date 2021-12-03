use anyhow::{Context, Result};
use api_metadata::api::contracts::events::Instantiated;
use api_metadata::api::system::events::ExtrinsicSuccess;
use api_metadata::api::{DefaultConfig, RuntimeApi};
use ink_env::call::{ExecutionInput, Selector};
use jsonrpsee_types::to_json_value;
use log::debug;
use pallet_contracts_primitives::ContractExecResult;
use parity_scale_codec::Encode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sp_keyring::AccountKeyring;
use sp_rpc::number::NumberOrHex;
use std::str::FromStr;
use subxt::{
    extrinsic::PairSigner,
    sp_core::{crypto::AccountId32, Bytes},
    sp_runtime::{
        traits::{BlakeTwo256, Hash},
        MultiAddress,
    },
};
use test_common::test_base_context::TestBaseContext;
use test_context::{test_context, AsyncTestContext};

const FLIPPER_CONTRACT_PATH: &str = "../contract/target/ink/flipper.wasm";
const ENDOWMENT: u128 = 1_111_000_000_000_000;
const GAS_LIMIT: u64 = 200_000_000_000;

static SETUP_ONCE: std::sync::Once = std::sync::Once::new();

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CallRequest {
    origin: AccountId32,
    dest: AccountId32,
    value: NumberOrHex,
    gas_limit: NumberOrHex,
    input_data: Bytes,
}

struct TestContractContext {
    test_base_context: TestBaseContext,
    contract_address: AccountId32,
}

#[async_trait::async_trait]
impl AsyncTestContext for TestContractContext {
    async fn setup() -> Self {
        SETUP_ONCE.call_once(|| {
            env_logger::init();
        });

        let test_base_context = TestBaseContext::setup().await;
        let mut test_contract_context = TestContractContext::new(test_base_context);

        test_contract_context
            .instantiate_flipper_contract()
            .await
            .expect("Failed to instantiate Flipper contract");

        test_contract_context
    }

    async fn teardown(self) {
        self.test_base_context.teardown().await;
    }
}

impl TestContractContext {
    fn new(test_base_context: TestBaseContext) -> Self {
        Self {
            test_base_context,
            contract_address: Default::default(),
        }
    }

    fn api(&self) -> RuntimeApi<DefaultConfig> {
        self.test_base_context.api()
    }

    async fn instantiate_flipper_contract(&mut self) -> Result<()> {
        let code = std::fs::read(FLIPPER_CONTRACT_PATH)
            .context("Failed to read Flipper contract wasm file")?;
        let signer = PairSigner::new(AccountKeyring::Alice.pair());

        let mut constructor_selector: [u8; 4] = Default::default();
        constructor_selector.copy_from_slice(&BlakeTwo256::hash(b"default")[0..4]);

        let constructor_selector = ExecutionInput::new(Selector::new(constructor_selector));
        let salt: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>();
        let result = self
            .api()
            .tx()
            .contracts()
            .instantiate_with_code(
                ENDOWMENT,
                GAS_LIMIT,
                code,
                constructor_selector.encode(),
                salt.encode(),
            )
            .sign_and_submit_then_watch(&signer)
            .await
            .context("Failed to instantiate_with_code Flipper contract")?;

        let instantiated = result
            .find_event::<Instantiated>()
            .context("Failed to decoe data to Instantiated type")?
            .context("Failed to find a Instantiated event")?;
        result
            .find_event::<ExtrinsicSuccess>()
            .context("Failed to decode data to ExtrinsicSuccess type")?
            .context("Failed to find a ExtrinsicSuccess event")?;

        debug!("Contract address: {}", instantiated.contract);

        self.contract_address = instantiated.contract;

        Ok(())
    }

    async fn call_get(&self) -> Result<ContractExecResult> {
        let mut get_selector: [u8; 4] = Default::default();
        get_selector.copy_from_slice(&BlakeTwo256::hash(b"get")[0..4]);
        let get_selector = ExecutionInput::new(Selector::new(get_selector));

        let call = CallRequest {
            origin: AccountKeyring::Alice.to_account_id(),
            dest: self.contract_address.clone(),
            value: sp_rpc::number::NumberOrHex::Number(0),
            gas_limit: GAS_LIMIT.into(),
            input_data: get_selector.encode().into(),
        };

        let params = &[to_json_value(call)?];

        let data = self
            .test_base_context
            .rpc()
            .client
            .request("contracts_call", params)
            .await?;
        Ok(data)
    }

    async fn call_flip(&self) -> Result<()> {
        let signer = PairSigner::new(AccountKeyring::Alice.pair());

        let mut flip_selector: [u8; 4] = Default::default();
        flip_selector.copy_from_slice(&BlakeTwo256::hash(b"flip")[0..4]);
        let flip_selector = ExecutionInput::new(Selector::new(flip_selector));

        let result = self
            .api()
            .tx()
            .contracts()
            .call(
                MultiAddress::Id(self.contract_address.clone()),
                0,
                GAS_LIMIT,
                flip_selector.encode(),
            )
            .sign_and_submit_then_watch(&signer)
            .await
            .context("Failed to call flip() method on Flipper contract")?;

        result
            .find_event::<api_metadata::api::system::events::ExtrinsicSuccess>()
            .context("Failed to decode data to ExtrinsicSuccess type")?
            .context("Failed to find a ExtrinsicSuccess event")?;

        debug!("call_flip() result {:?}", result);

        Ok(())
    }
}

#[test_context(TestContractContext)]
#[tokio::test]
async fn default_constructor_set_value_to_0(ctx: &mut TestContractContext) -> Result<()> {
    let response: ContractExecResult = ctx.call_get().await?;

    debug!("call_get response = {:?}", response);
    assert!(response
        .result
        .as_ref()
        .expect("call_get failed")
        .is_success());
    assert_eq!(
        Bytes::from_str("0x0")?,
        response.result.as_ref().expect("call_get failed").data
    );

    Ok(())
}

#[test_context(TestContractContext)]
#[tokio::test]
async fn flip_changes_value(ctx: &mut TestContractContext) -> Result<()> {
    let response: ContractExecResult = ctx.call_get().await?;

    debug!("call_get response = {:?}", response);
    assert!(response
        .result
        .as_ref()
        .expect("call_get failed")
        .is_success());
    assert_eq!(
        Bytes::from_str("0x0")?,
        response.result.as_ref().expect("call_get failed").data
    );

    ctx.call_flip().await?;

    let response: ContractExecResult = ctx.call_get().await?;

    debug!("call_get response = {:?}", response);
    assert!(response
        .result
        .as_ref()
        .expect("call_get failed")
        .is_success());
    assert_eq!(
        Bytes::from_str("0x1")?,
        response.result.as_ref().expect("call_get failed").data
    );

    Ok(())
}
