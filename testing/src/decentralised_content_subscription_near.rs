mod utils;
use utils::utils::*;
mod contract_call_tester;
use contract_call_tester::contract_call_tester::*;
use serde_json::json;
use workspaces::prelude::*;

const DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH: &str = "contracts/res/decentralised_content_subscription_near.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // build the contract if not yet built
    let wasm = get_wasm(DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH);
    
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
    // payment_interval for creators profile: represents how many pieces
    // of content a single payment grants a subscriber access to
    let payment_interval = "3";
    // content to be added to the creators profile
    let content = "https://www.youtube.com/watch?v=MddGbXgIt2E".to_owned();
    // date to be used as a key for creators content
    let content_id = "1".to_owned();

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
    match test_contract_call(&creator, &worker_creator,&contract,"add_profile",
        json!({
            "account_id": &creator.id().to_owned(),
            "profile_type": "creator",
            "cost": creator_profile_cost,
            "payment_interval": payment_interval
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
    match test_contract_call(&creator, &worker_creator, &contract, "get_profile",
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
    match test_contract_call(&creator, &worker_creator, &contract, "add_content",
        json!({
            "date": &content_id,
            "content": &content
        }),
        "", false
    ).await {
        Ok(_) => println!("add_content with creator: passed"),
        Err(_) => println!("add_content with creator: failed")
    };
    
    // tests get_content method of contract - called by profile creator
    test_contract_call(&creator, &worker_creator, &contract, "get_content",
        json!({
            "creator_address": creator.id(),
            "date": &content_id
        }),
        &content, true
    ).await.expect("error when getting content");
    
    // tests get_content method of contract - expects a panic as called by non-subscriber
    match test_contract_call(&consumer, &worker_consumer, &contract, "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": &content_id
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
    match test_contract_call(&consumer, &worker_consumer, &contract, "subscribe",
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
    test_contract_call(&consumer, &worker_consumer, &contract, "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": &content_id
        }),
        &content, true
    ).await.expect("error when getting content with subscriber");
    
    // subscribe with insufficient funds
    // update cost to above test account balance
    match test_contract_call(&creator, &worker_creator, &contract, "update_cost",
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
    test_contract_call(&creator, &worker_creator, &contract, "get_cost",
        json!({}), "201", true
    ).await.expect("error when getting getting cost");

    match test_contract_call(&consumer_low_balace, &worker_consumer_low_balance, &contract, "subscribe",
        json!({"creator_address": &creator.id().to_owned()}), "", true
    ).await {
        Ok(_) => println!("subscribe with consumer with low balance: failed"),
        Err(_) => println!("subscribe with consumer with low balance: passed")
    };

    // update cost to below test account balance
    match test_contract_call(&creator, &worker_creator, &contract, "update_cost",
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
    match test_contract_call(&consumer_low_balace, &worker_consumer_low_balance, &contract, "subscribe",
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

    // add more content such that consumer can't access the latest content
    for content in [("2", "content 2"), ("3", "content 3"), ("4", "content 4"), ("5", "content 5")] {
        match test_contract_call(&creator, &worker_creator, &contract, "add_content",
            json!({
                "date": content.0,
                "content": content.1
            }),
            "", false
        ).await {
            Ok(_) => println!("add_content_{} with creator: passed", content.0),
            Err(_) => println!("add_content_{} with creator: failed", content.0)
        };
    }

    // get content that consumer should not have access too: expects a panicjjj
    match test_contract_call(&consumer, &worker_consumer, &contract, "get_content",
        json!({
            "creator_address": &creator.id().to_owned(),
            "date": "5"
        }),
        "content 5", true
    ).await {
        Ok(_) => println!("get_content when no access: failed"),
        Err(error) => println!("get_content when no access: passed {}", error)
    };

    remove_near_credentials();

    Ok(())
}