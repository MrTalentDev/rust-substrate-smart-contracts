use anyhow::{Context, Result};
use portpicker::Port;
use std::process::{Child, Command};

pub struct TestNodeContext {
    node_process: Child,
    ws_port: Port,
}

impl Drop for TestNodeContext {
    fn drop(&mut self) {
        self.shutdown()
            .expect("Failed to shutdown Substrate Node process");
    }
}

impl TestNodeContext {
    pub fn new() -> Result<Self> {
        let p2p_port: Port = portpicker::pick_unused_port().context("No P2P ports free")?;
        let ws_port: Port = portpicker::pick_unused_port().context("No WS ports free")?;

        let node_process = Command::new("substrate-contracts-node")
            .arg("--dev")
            .arg("--ws-external")
            .arg(format!("--port={}", p2p_port))
            .arg(format!("--ws-port={}", ws_port))
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to spawn Substrate Node process p2p_port {} ws_port {}",
                    p2p_port, ws_port
                )
            })?;

        Ok(Self {
            node_process,
            ws_port,
        })
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.node_process
            .kill()
            .context("Failed to kill Substrate Node process")
    }

    pub fn get_ws_port(&self) -> Port {
        self.ws_port
    }
}
