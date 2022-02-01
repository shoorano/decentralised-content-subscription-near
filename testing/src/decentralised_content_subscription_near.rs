use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Account, Worker, Contract, DevNetwork};

const DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH: &str = "contracts/target/wasm32-unknown-unknown/release/decentralised_content_subscription_near.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker_deployer = workspaces::testnet();
    let worker_creator = workspaces::testnet();
    let worker_consumer = workspaces::testnet();
    
    let wasm = std::fs::read(DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH)?;

    let contract = worker_deployer.dev_deploy(wasm)
        .await
        .expect("failed to deploy contract");

    let creator = worker_creator.dev_create_account()
        .await
        .expect("failed to create creator account");
    let content = "https://www.youtube.com/watch?v=MddGbXgIt2E".to_owned();
    let creator_profile_cost = "1".to_owned();

    let consumer = worker_consumer.dev_create_account()
        .await
        .expect("failed to create consumer account");

    add_profile(&creator, &worker_creator, &contract, &creator_profile_cost)
        .await
        .expect("error when adding profile");
    
    add_content(&creator, &worker_creator, &contract, &content)
        .await
        .expect("error when adding content");
        
    get_content(&creator, &creator, &worker_creator, &contract, &content)
        .await
        .expect("error when getting content");
    
    match get_content(&consumer, &creator, &worker_consumer, &contract, &content).await {
        Ok(_) => println!("get_content with none subscriber: failed"),
        Err(_) => println!("get_content with none subscriber: passed")
    }

    // tests to add
    // subscribe then get_content
    // subscribe with low balance

    Ok(())
    }


async fn add_profile(creator: &Account, worker_creator: &Worker<impl DevNetwork>, contract: &Contract, cost: &str) -> anyhow::Result<()> {
    creator
        .call(
            &worker_creator,
            contract.id().to_owned(),
            "add_profile"
        )
        .args_json(json!({
            "account_id": &creator.id().to_owned(),
            "profile_type": "creator",
            "cost": cost
        }))?
        .transact()
        .await?;

    println!("add_profile test: passed");

    Ok(())
}

async fn add_content(creator: &Account, worker: &Worker<impl DevNetwork>, contract: &Contract, content: &str) -> anyhow::Result<()> {
    creator
        .call(
            &worker,
            contract.id().to_owned(),
            "add_content"
        )
        .args_json(json!({
            "date": "31-01-2022",
            "content": content
        }))?
        .transact()
        .await?;

    println!("add_content test: passed");

    Ok(())
}

async fn get_content(caller: &Account, creator: &Account, worker: &Worker<impl DevNetwork>, contract: &Contract, content: &str) -> anyhow::Result<()> {
    let result = caller
        .call(
            &worker,
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
        Ok(result) => {
            if (result == content.to_owned()) == true {
            println!("get_content test: passed");
            }
        },
        Err(error) => return Err(error)
    }
    Ok(())
}