
use subxt::{
    ClientBuilder,
};

use api_metadata::api::DefaultConfig;
use api_metadata::api::RuntimeApi;

#[tokio::test]
async fn storage() -> Result<(), Box<dyn std::error::Error>> {
    let api = ClientBuilder::new().set_url("ws://substrate-contracts-node:9944")
        .build()
        .await?
        .to_runtime_api::<RuntimeApi<DefaultConfig>>();

    let mut iter = api.storage().system().account_iter(None).await?;

    while let Some((key, account)) = iter.next().await? {
        println!("{}: {}", hex::encode(key), account.data.free);
    }

    Ok(())
}


#[tokio::test]
async fn fetch_remote() -> Result<(), Box<dyn std::error::Error>> {
    let api = ClientBuilder::new()
        .set_url("ws://substrate-contracts-node:9944")
        .build()
        .await?
        .to_runtime_api::<RuntimeApi<DefaultConfig>>();

    let block_number = 1;

    let block_hash = api
        .client
        .rpc()
        .block_hash(Some(block_number.into()))
        .await?;

    if let Some(hash) = block_hash {
        println!("Block hash for block number {}: {}", block_number, hash);
    } else {
        println!("Block number {} not found.", block_number);
    }

    Ok(())
}