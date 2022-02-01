use serde_json::json;
use workspaces::prelude::*;

const DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH: &str = "contracts/target/wasm32-unknown-unknown/release/decentralised_content_subscription_near.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker_deployer = workspaces::testnet();
    let worker_creator = workspaces::testnet();
    
    let wasm = std::fs::read(DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH)?;

    let contract = worker_deployer.dev_deploy(wasm)
        .await
        .expect("failed to deploy contract");

    let creator = worker_creator.dev_create_account()
        .await
        .expect("failed to create creator account");

    let outcome_add_profile = creator
        .call(
            &worker_creator,
            contract.id().to_owned(),
            "add_profile"
        )
        .args_json(json!({
            "account_id": &creator.id().to_owned(),
            "profile_type": "creator",
            "cost": "100000000000000000000"
        }))?
        .transact()
        .await?;

    println!("result add profile: {:?}", outcome_add_profile);

    let outcome_add_content = creator
        .call(
            &worker_creator,
            contract.id().to_owned(),
            "add_content"
        )
        .args_json(json!({
            "date": "31-01-2022",
            "content": "https://www.youtube.com/watch?v=MddGbXgIt2E"
        }))?
        .transact()
        .await?;

    println!("result get profile: {:?}", outcome_add_content);

    let result = creator
        .call(
            &worker_creator,
            contract.id().to_owned(),
            "get_content"
        )
        .args_json(json!({
            "creator_address": creator.id().to_owned(),
            "date": "31-01-2022",
        }))?
        .transact()
        .await?
        .json::<String>();
    
    match result {
        Ok(content) => println!("result: {:?}", content),
        Err(error) => println!("error: {:?}", error)
    }
    Ok(())
}