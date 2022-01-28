# build contract wasm
cargo build --target wasm32-unknown-unknown --release

# deploy contract with dev test account to testnet
near dev-deploy target/wasm32-unknown-unknown/release/decentralised_patreon.wasm

# call add_profile with real testnet account
near call dev-1643384759160-77197182459258 add_profile '{"account_id": "bowtiedgon.testnet", "profile_type": "creator", "cost": "1000000000000000000000"}' --account-id dev-1643384759160-77197182459258

# test get_profile - expect no return value but no errors
near call dev-1643384759160-77197182459258 get_profile '{"account_id": "bowtiedgon.testnet"}' --account-id dev-1643384759160-77197182459258

# test adding content to real testnet accounts profile
near call dev-1643384759160-77197182459258 add_content '{"date": "28-01-2022", "content": "This is a test of adding content"}' --account-id bowtiedgon.testnet

# call get_content from profile owners address - expects content returned as string
near call dev-1643384759160-77197182459258 get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id bowtiedgon.testnet

# call with none subscriber - expect panic and error
near call dev-1643384759160-77197182459258 get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id dev-1643384759160-77197182459258

# subscribe with dev account
near call dev-1643384759160-77197182459258 subscribe '{"creator_address": "bowtiedgon.testnet"}' --account-id dev-1643384759160-77197182459258

# call with dev account which is now subscribed - expect content returned
near call dev-1643384759160-77197182459258 get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id dev-1643384759160-77197182459258