use anyhow::{Context, Result};
use api_metadata::api::{DefaultConfig, RuntimeApi};
use ink_env::call::{ExecutionInput, Selector};
use parity_scale_codec::Encode;
use rand::Rng;
use sp_keyring::AccountKeyring;
use subxt::{
    extrinsic::PairSigner,
    sp_runtime::traits::{BlakeTwo256, Hash},
};
use test_common::test_base_context::TestBaseContext;
use test_context::{test_context, AsyncTestContext};

const FLIPPER_CONTRACT_PATH: &str = "../contract/target/ink/flipper.wasm";
const ENDOWMENT: u128 = 1_111_000_000_000_000;
const GAS_LIMIT: u64 = 200_000_000_000;

struct TestContractContext {
    test_base_context: TestBaseContext,
}

#[async_trait::async_trait]
impl AsyncTestContext for TestContractContext {
    async fn setup() -> Self {
        let test_base_context = TestBaseContext::setup().await;

        let test_contract_context = Self { test_base_context };
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
    fn api(&self) -> RuntimeApi<DefaultConfig> {
        self.test_base_context.api()
    }

    async fn instantiate_flipper_contract(&self) -> Result<()> {
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
            .find_event::<api_metadata::api::contracts::events::Instantiated>()
            .context("Failed to decoe data to Instantiated type")?
            .context("Failed to find a Instantiated event")?;
        result
            .find_event::<api_metadata::api::system::events::ExtrinsicSuccess>()
            .context("Failed to decoe data to ExtrinsicSuccess type")?
            .context("Failed to find a ExtrinsicSuccess event")?;

        println!("Contract address: {}", instantiated.contract);

        Ok(())
    }
}

#[test_context(TestContractContext)]
#[tokio::test]
async fn test_works(ctx: &mut TestContractContext) {}
