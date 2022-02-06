use std::process::Command;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Account, Worker, Contract, DevNetwork};

const DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH: &str = "contracts/res/decentralised_content_subscription_near.wasm";

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
    let worker_consumer_low_balance = workspaces::testnet();
    
    // deploy the contract
    let contract = worker_deployer.dev_deploy(wasm)
        .await
        .expect("failed to deploy contract");

    // create an Account struct for the creator profile type
    let creator = worker_creator.dev_create_account()
        .await
        .expect("failed to create creator account");
    // cost to subscribe to creators profile in NEAR
    let creator_profile_cost = "1";
    // content to be added to the creators profile
    let content = "https://www.youtube.com/watch?v=MddGbXgIt2E".to_owned();
    // date to be used as a key for creators content
    let content_date = "31-01-2022".to_owned();

    // create an Account struct for a consumer who will subscribe to creators profile
    let consumer = worker_consumer.dev_create_account()
        .await
        .expect("failed to create consumer account");
    
    // create an Account struct for a consumer who will subscribe to creators profile
    // the creator will increase the cost to above the balance of test accounts
    let consumer_low_balace = worker_consumer_low_balance.dev_create_account()
        .await
        .expect("failed to create consumer_low_balace account");

    // tests add_profile method of contract
    match test_contract_method(&creator, &worker_creator,&contract,"add_profile",
        json!({
            "account_id": &creator.id().to_owned(),
            "profile_type": "creator",
            "cost": creator_profile_cost
        }),
        "", false
    ).await {
        Ok(_) => println!("add_profile with creator: passed"),
        Err(error) => {
            println!("add_profile with creator: failed");
            println!("error: {}", error);
        }
    };

    // test get_profile method
    match test_contract_method(&creator, &worker_creator, &contract, "get_profile",
        json!({
            "account_id": &creator.id().to_owned()
        }),
        "", false
    ).await {
        Ok(_) => println!("get_profile with creator: passed"),
        Err(error) => {
            println!("get_profile with creator: failed");
            println!("error: {}", error);
        }
    };
    
    // tests add_content method of contract        
    match test_contract_method(&creator, &worker_creator, &contract, "add_content",
        json!({
            "date": &content_date,
            "content": &content
        }),
        "", false
    ).await {
        Ok(_) => println!("add_content with creator: passed"),
        Err(_) => println!("add_content with creator: failed")
    };
    
    // tests get_content method of contract - called by profile creator
    test_contract_method(&creator, &worker_creator, &contract, "get_content",
        json!({
            "creator_address": creator.id(),
            "date": &content_date
        }),
        &content, true
    ).await.expect("error when getting content");
    
    // tests get_content method of contract - expects a panic as called by non-subscriber
    match test_contract_method(&consumer, &worker_consumer, &contract, "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": &content_date
        }),
        &content, true
    ).await {
        Ok(_) => println!("get_content with none subscriber: failed"),
        Err(error) => {
            println!("get_content with none subscriber: passed");
            println!("error: {}", error);
        }
    };
    
    // subscribe then get_content
    // subscribe
    match test_contract_method(&consumer, &worker_consumer, &contract, "subscribe",
        json!({
            "creator_address": &creator.id().to_owned()
        }),
        "", false
    ).await {
        Ok(_) => println!("subscribe with consumer: passed"),
        Err(error) => {
            println!("subscribe with consumer: failed");
            println!("error: {}", error);
        }
    };

    // get_content now that consumer has subscribed to creator - expects content to be returned
    test_contract_method(&consumer, &worker_consumer, &contract, "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": &content_date
        }),
        &content, false
    ).await.expect("error when getting content with subscriber");
    
    // subscribe with insufficient funds
    // update cost to above test account balance
    match test_contract_method(&creator, &worker_creator, &contract, "update_cost",
        json!({
            "cost": "201"
        }),
        "", false
    ).await {
        Ok(_) => println!("update_cost with creator: passed"),
        Err(error) => {
            println!("update_cost with creator: failed");
            println!("error: {}", error);
        }
    };

    // check cost updated
    test_contract_method(&creator, &worker_creator, &contract, "get_cost",
        json!({}), "201", true
    ).await.expect("error when getting getting cost");

    match test_contract_method(&consumer_low_balace, &worker_consumer_low_balance, &contract, "subscribe",
        json!({"creator_address": &creator.id().to_owned()}), "", true
    ).await {
        Ok(_) => println!("subscribe with consumer with low balance: failed"),
        Err(_) => println!("subscribe with consumer with low balance: passed")
    };

    // update cost to below test account balance
    match test_contract_method(&creator, &worker_creator, &contract, "update_cost",
        json!({
            "cost": "2"
        }),
        "", false
    ).await {
        Ok(_) => println!("update_cost with creator: passed"),
        Err(error) => {
            println!("update_cost with creator: failed");
            println!("error: {}", error);
        }
    };

    // subscribe at new lower cost
    match test_contract_method(&consumer_low_balace, &worker_consumer_low_balance, &contract, "subscribe",
        json!({
            "creator_address": &creator.id().to_owned()
        }),
        "", false
    ).await {
        Ok(_) => println!("subscribe with consumer now that cost lowered: passed"),
        Err(error) => {
            println!("subscribe with consumer now that cost lowered: failed");
            println!("error: {}", error);
        }
    };

    test_contract_method(&creator, &worker_creator, &contract, ".data",
        json!({}),
        "", true
    ).await.expect("no method named data");



    remove_near_credentials();
    Ok(())
}

fn build_contract() {
    let output = Command::new("sh")
                .arg("contracts/build.sh")
                .output()
                .expect("failed to execute process");
    println!("Building contract output: {:?}", output);
}

fn remove_near_credentials() {
    let output = Command::new("rm")
                .arg("-rf")
                .arg(".near-credentials/")
                .output()
                .expect("failed to execute process");
    println!("Removing near credentials output: {:?}", output);
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
    
    // for methods which return non-serialized data
    if !result_is_serializable {
        return Ok(());
    }

    // parse result to json
    match result.json::<String>() {
        Ok(result) => {
            if (result == expected_result.to_owned()) == true {
            println!("{} test: passed", method);
            } else {
                println!("{} test: failed", method);
                println!("left: {} != right: {}", result, expected_result);
            }
        },
        Err(error) => return Err(error)
    }
    Ok(())
}
