use anyhow::{Context, Result};
use log::debug;
use pallet_contracts_primitives::ContractExecResult;
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use subxt::sp_core::Bytes;
use test_common::test_contract_context::TestContractContext;
use test_context::{test_context, AsyncTestContext};

const FLIPPER_CONTRACT_PATH: &str = "../contract/target/ink/flipper.wasm";
const ENDOWMENT: u128 = 1_111_000_000_000_000;
const GAS_LIMIT: u64 = 200_000_000_000;

static SETUP_ONCE: std::sync::Once = std::sync::Once::new();

struct TestFlipperContractContext {
    test_contract_context: TestContractContext,
}

#[async_trait::async_trait]
impl AsyncTestContext for TestFlipperContractContext {
    async fn setup() -> Self {
        SETUP_ONCE.call_once(|| {
            env_logger::init();
        });

        let mut test_contract_context = TestContractContext::setup().await;

        let constructor_data = TestContractContext::create_exec_input("default")
            .expect("Failed to create constructor data");

        test_contract_context
            .instantiate_contract(
                ENDOWMENT,
                GAS_LIMIT,
                FLIPPER_CONTRACT_PATH,
                &constructor_data,
                AccountKeyring::Alice,
            )
            .await
            .expect("Failed to instantiate Flipper contract");

        TestFlipperContractContext {
            test_contract_context,
        }
    }

    async fn teardown(self) {
        self.test_contract_context.teardown().await;
    }
}

impl TestFlipperContractContext {
    async fn call_get(&self) -> Result<ContractExecResult> {
        let call_data = TestContractContext::create_exec_input("get")
            .context("Failed to create get function data")?;
        self.test_contract_context
            .call(AccountKeyring::Alice, 0, GAS_LIMIT, &call_data)
            .await
    }

    async fn call_flip(&self) -> Result<()> {
        let call_data = TestContractContext::create_exec_input("flip")
            .context("Failed to create flip function data")?;
        self.test_contract_context
            .call_ext(AccountKeyring::Alice, 0, GAS_LIMIT, &call_data)
            .await
    }
}

#[test_context(TestFlipperContractContext)]
#[tokio::test]
async fn default_constructor_set_value_to_0(ctx: &mut TestFlipperContractContext) -> Result<()> {
    let response: ContractExecResult = ctx
        .call_get()
        .await
        .context("Failed to call get method on Flipper contract")?;

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

#[test_context(TestFlipperContractContext)]
#[tokio::test]
async fn flip_changes_value(ctx: &mut TestFlipperContractContext) -> Result<()> {
    let response: ContractExecResult = ctx
        .call_get()
        .await
        .context("Failed to call get method on Flipper contract")?;

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

    ctx.call_flip()
        .await
        .context("call_flip failed")
        .context("Failed to call flip method on Flipper contract")?;

    let response: ContractExecResult = ctx
        .call_get()
        .await
        .context("Failed to call get method on Flipper contract")?;

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
