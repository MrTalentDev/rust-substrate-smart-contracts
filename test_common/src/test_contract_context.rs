use crate::test_base_context::TestBaseContext;
use anyhow::{Context, Result};
use api_metadata::api::contracts::events::Instantiated;
use api_metadata::api::system::events::ExtrinsicSuccess;
use api_metadata::api::{DefaultConfig, RuntimeApi};
use ink_env::call::{utils::EmptyArgumentList, ExecutionInput, Selector};
use jsonrpsee_types::to_json_value;
use log::debug;
use pallet_contracts_primitives::ContractExecResult;
use parity_scale_codec::Encode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sp_keyring::AccountKeyring;
use sp_rpc::number::NumberOrHex;
use subxt::{
    extrinsic::PairSigner,
    sp_core::{crypto::AccountId32, Bytes},
    sp_runtime::{
        traits::{BlakeTwo256, Hash},
        MultiAddress,
    },
};
use test_context::AsyncTestContext;

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

pub struct TestContractContext {
    pub test_base_context: TestBaseContext,
    pub contract_address: Option<AccountId32>,
}

#[async_trait::async_trait]
impl AsyncTestContext for TestContractContext {
    async fn setup() -> Self {
        let test_base_context = TestBaseContext::setup().await;

        TestContractContext::new(test_base_context)
    }

    async fn teardown(self) {
        self.test_base_context.teardown().await;
    }
}

impl TestContractContext {
    fn new(test_base_context: TestBaseContext) -> Self {
        Self {
            test_base_context,
            contract_address: None,
        }
    }

    pub fn api(&self) -> RuntimeApi<DefaultConfig> {
        self.test_base_context.api()
    }

    pub fn create_exec_input(method: &str) -> Result<ExecutionInput<EmptyArgumentList>> {
        Ok(ExecutionInput::new(Selector::new(
            BlakeTwo256::hash(method.as_bytes())[0..4].try_into()?,
        )))
    }

    pub async fn instantiate_contract<T>(
        &mut self,
        endowment: u128,
        gas_limit: u64,
        contract_path: &str,
        constructor_data: &ExecutionInput<T>,
        account: AccountKeyring,
    ) -> Result<()>
    where
        T: parity_scale_codec::Encode,
    {
        let code = std::fs::read(contract_path).context("Failed to read contract wasm file")?;
        let signer = PairSigner::new(account.pair());
        let salt: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>();

        let result = self
            .api()
            .tx()
            .contracts()
            .instantiate_with_code(
                endowment,
                gas_limit,
                code,
                constructor_data.encode(),
                salt.encode(),
            )
            .sign_and_submit_then_watch(&signer)
            .await
            .context("Failed to instantiate contract")?;

        let instantiated = result
            .find_event::<Instantiated>()
            .context("Failed to decode event data to Instantiated type")?
            .context("Failed to find a Instantiated event")?;
        result
            .find_event::<ExtrinsicSuccess>()
            .context("Failed to decode event data to ExtrinsicSuccess type")?
            .context("Failed to find a ExtrinsicSuccess event")?;

        debug!("Contract address: {}", instantiated.contract);

        self.contract_address = Some(instantiated.contract);

        Ok(())
    }

    pub async fn call<T>(
        &self,
        origin: AccountKeyring,
        value: u64,
        gas_limit: u64,
        input_data: &ExecutionInput<T>,
    ) -> Result<ContractExecResult>
    where
        T: parity_scale_codec::Encode,
    {
        let call = CallRequest {
            origin: origin.to_account_id(),
            dest: self
                .contract_address
                .as_ref()
                .context("contract is not instantiated")?
                .clone(),
            value: value.into(),
            gas_limit: gas_limit.into(),
            input_data: input_data.encode().into(),
        };

        let params = &[to_json_value(call).context("Failed to convert to JSON data")?];

        let result = self
            .test_base_context
            .rpc()
            .client
            .request("contracts_call", params)
            .await
            .context("Failed to call call() method on contract")?;

        debug!("call() result {:?}", result);

        Ok(result)
    }

    pub async fn call_ext<T>(
        &self,
        account: AccountKeyring,
        value: u64,
        gas_limit: u64,
        input_data: &ExecutionInput<T>,
    ) -> Result<()>
    where
        T: parity_scale_codec::Encode,
    {
        let result = self
            .api()
            .tx()
            .contracts()
            .call(
                MultiAddress::Id(
                    self.contract_address
                        .as_ref()
                        .context("contract is not instantiated")?
                        .clone(),
                ),
                value.into(),
                gas_limit,
                input_data.encode(),
            )
            .sign_and_submit_then_watch(&PairSigner::new(account.pair()))
            .await
            .context("Failed to call flip() method on contract")?;

        result
            .find_event::<ExtrinsicSuccess>()
            .context("Failed to decode event data to ExtrinsicSuccess type")?
            .context("Failed to find a ExtrinsicSuccess event")?;

        debug!("call_ext() result {:?}", result);

        Ok(())
    }
}
