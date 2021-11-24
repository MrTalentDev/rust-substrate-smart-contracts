use crate::consts::NODE_HOST;
use anyhow::{Context, Result};
use api_metadata::api::DefaultConfig;
use portpicker::Port;
use subxt::{Client, ClientBuilder};

pub struct TestClientContext {
    client: Client<DefaultConfig>,
}

impl TestClientContext {
    pub async fn new(ws_port: Port) -> Result<Self> {
        let client = ClientBuilder::new()
            .set_url(format!("ws://{}:{}", NODE_HOST, ws_port))
            .build()
            .await
            .context("Failed to build client")?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &Client<DefaultConfig> {
        &self.client
    }
}
