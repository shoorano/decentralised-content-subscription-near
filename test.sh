echo build contract wasm
echo
cargo build --target wasm32-unknown-unknown --release
echo

echo deploy contract with dev test account to testnet
echo
near dev-deploy target/wasm32-unknown-unknown/release/decentralised_patreon.wasm
echo

echo call add_profile with real testnet account
echo
near call dev-1643384759160-77197182459258 add_profile '{"account_id": "bowtiedgon.testnet", "profile_type": "creator", "cost": "1000000000000000000000"}' --account-id dev-1643384759160-77197182459258
echo

echo test get_profile - expect no return value but no errors
echo
near call dev-1643384759160-77197182459258 get_profile '{"account_id": "bowtiedgon.testnet"}' --account-id dev-1643384759160-77197182459258
echo

echo test adding content to real testnet accounts profile
echo
near call dev-1643384759160-77197182459258 add_content '{"date": "28-01-2022", "content": "This is a test of adding content"}' --account-id bowtiedgon.testnet
echo

echo call get_content from profile owners address - expects content returned as string
echo
near call dev-1643384759160-77197182459258 get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id bowtiedgon.testnet
echo

echo call with none subscriber - expect panic and error
echo
near call dev-1643384759160-77197182459258 get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id dev-1643384759160-77197182459258
echo

echo subscribe with dev account
echo
near call dev-1643384759160-77197182459258 subscribe '{"creator_address": "bowtiedgon.testnet"}' --account-id dev-1643384759160-77197182459258
echo

echo call with dev account which is now subscribed - expect content returned
echo
near call dev-1643384759160-77197182459258 get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id dev-1643384759160-77197182459258
echo