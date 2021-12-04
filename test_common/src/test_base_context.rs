use crate::test_client_context::TestClientContext;
use crate::test_node_context::TestNodeContext;
use api_metadata::api::{DefaultConfig, RuntimeApi};
use std::{thread, time};
use subxt::rpc::Rpc;
use test_context::AsyncTestContext;

const RECONNECT_WAIT_TIME: time::Duration = time::Duration::from_secs(1);
const MAX_RECONNECTS: u32 = 10;

pub struct TestBaseContext {
    _test_node_context: TestNodeContext,
    test_client_context: TestClientContext,
}

#[async_trait::async_trait]
impl AsyncTestContext for TestBaseContext {
    async fn setup() -> Self {
        let test_node_context = TestNodeContext::new().expect("Couldn't create TestNodeContext");

        let ws_port = test_node_context.get_ws_port();
        let mut counter = 0;
        let test_client_context = loop {
            thread::sleep(RECONNECT_WAIT_TIME);
            let result = TestClientContext::new(ws_port).await;

            match result {
                Ok(client) => break client,
                Err(_) => {
                    if counter < MAX_RECONNECTS {
                        counter += 1;
                        continue;
                    }
                    result.expect("Couldn't connect to node");
                }
            }
        };

        Self {
            _test_node_context: test_node_context,
            test_client_context,
        }
    }
}

impl TestBaseContext {
    pub fn api(&self) -> RuntimeApi<DefaultConfig> {
        self.test_client_context.client().clone().to_runtime_api()
    }

    pub fn rpc(&self) -> &Rpc<DefaultConfig> {
        self.test_client_context.client().rpc()
    }
}
