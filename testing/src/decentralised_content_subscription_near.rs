use std::process::Command;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Account, Worker, Contract, DevNetwork};

const DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH: &str = "target/wasm32-unknown-unknown/release/decentralised_content_subscription_near.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build the contract if not yet built
    let wasm = match std::fs::read(DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH) {
        Err(_) => {
            build_contract();  
            std::fs::read(DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH)?
        },
        Ok(wasm) => {
            println!("Contract is already built, returning wasm");
            wasm
        }
    };
    
    // build requirements for the testing methods
    let worker_deployer = workspaces::testnet();
    let worker_creator = workspaces::testnet();
    let worker_consumer = workspaces::testnet();
    
    // deploy the contract
    let contract = worker_deployer.dev_deploy(wasm)
        .await
        .expect("failed to deploy contract");

    // create an Account struct for the creator profile type
    let creator = worker_creator.dev_create_account()
        .await
        .expect("failed to create creator account");
    // content to be added to the creators profile
    let content = "https://www.youtube.com/watch?v=MddGbXgIt2E".to_owned();
    // cost to subscribe to creators profile in NEAR
    let creator_profile_cost = "1";

    // create an Account struct for a consumer who will subscribe to creators profile
    let consumer = worker_consumer.dev_create_account()
        .await
        .expect("failed to create consumer account");

    // tests add_profile method of contract
    test_contract_method(
        &creator,
        &worker_creator,
        &contract,
        "add_profile",
        json!({
            "account_id": &creator.id().to_owned(),
            "profile_type": "creator",
            "cost": creator_profile_cost
        }),
        "",
        false
    ).await.expect("error when adding profile");

    // test get_profile method
    test_contract_method(
        &creator,
        &worker_creator,
        &contract,
        "get_profile",
        json!({
            "account_id": &creator.id().to_owned()
        }),
        "",
        false
    ).await.expect("error when getting profile");
    
    // tests add_content method of contract        
    test_contract_method(
        &creator,
        &worker_creator,
        &contract,
        "add_content",
        json!({
            "date": "31-01-2022",
            "content": &content
        }),
        "",
        false
    ).await.expect("error when adding content");
    
    // tests get_content method of contract - called by profile creator
    test_contract_method(
        &creator,
        &worker_creator,
        &contract,
        "get_content",
        json!({
            "creator_address": creator.id(),
            "date": "31-01-2022"
        }),
        &content,
        true
    ).await.expect("error when getting content");
    
    // tests get_content method of contract - expects a panic as called by non-subscriber
    match test_contract_method(
        &consumer,
        &worker_consumer,
        &contract,
        "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": "31-01-2022"
        }),
        &content,
        true
    ).await {
        Ok(_) => println!("get_content with none subscriber: failed"),
        Err(_) => println!("get_content with none subscriber: passed")
    }
    
    // subscribe then get_content
    // subscribe
    test_contract_method(
        &consumer,
        &worker_consumer,
        &contract,
        "subscribe",
        json!({
            "creator_address": &creator.id().to_owned()
        }),
        "",
        false
    ).await.expect("error when subscribing");

    // get_content now that consumer has subscribed to creator - expects content to be returned
    test_contract_method(
        &consumer,
        &worker_consumer,
        &contract,
        "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": "31-01-2022"
        }),
        &content,
        true
    ).await.expect("error when getting content with subscriber");
    
    // tests to add
    // subscribe with low balance

    Ok(())
}

fn build_contract() {
    let output = Command::new("sh")
                .arg("contracts/build.sh")
                .output()
                .expect("failed to execute process");
    println!("Building contract output: {:?}", output);
}

async fn test_contract_method(
    caller: &Account,
    worker: &Worker<impl DevNetwork>,
    contract: &Contract,
    method: &str,
    arguments: serde_json::Value,
    expected_result: &str,
    result_is_serializable: bool
) -> anyhow::Result<()> {
    println!("testing {} method", method);
    // make contract call
    let result = caller
        .call(
            &worker,
            contract.id().to_owned(),
            method
        )
        .args_json(arguments)?
        .transact()
        .await?;
    
    // for method
    if !result_is_serializable {
        return Ok(());
    }

    match result.json::<String>() {
        Ok(result) => {
            if (result == expected_result) == true {
            println!("get_content test: passed");
            }
        },
        Err(error) => return Err(error)
    }
    Ok(())
}
