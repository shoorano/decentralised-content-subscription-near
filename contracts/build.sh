echo build contract wasm
echo
cargo build --target wasm32-unknown-unknown --release
echo

# echo delete contract account
# echo
# near delete tester.bowtiedgon.testnet bowtiedgon.testnet
# echo

# echo creating tester account for deploying contract too
# echo
# near create-account tester.bowtiedgon.testnet --masterAccount bowtiedgon.testnet
# echo

# echo deploy contract with dev test account to testnet
# echo
# near deploy tester.bowtiedgon.testnet target/wasm32-unknown-unknown/release/decentralised_content_subscription_near.wasm
# echo

# echo call add_profile with real testnet account
# echo
# near call tester.bowtiedgon.testnet add_profile '{"account_id": "bowtiedgon.testnet", "profile_type": "creator", "cost": "1000000000000000000000"}' --account-id bowtiedgon.testnet
# echo

# echo test get_profile - expect no return value but no errors
# echo
# near call tester.bowtiedgon.testnet get_profile '{"account_id": "bowtiedgon.testnet"}' --account-id tester.bowtiedgon.testnet
# echo

# echo test adding content to real testnet accounts profile
# echo
# near call tester.bowtiedgon.testnet add_content '{"date": "28-01-2022", "content": "This is a test of adding content"}' --account-id bowtiedgon.testnet
# echo

# echo call get_content from profile owners address - expects content returned as string
# echo
# near call tester.bowtiedgon.testnet get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id bowtiedgon.testnet
# echo

# echo call with none subscriber - expect panic and error
# echo
# near call tester.bowtiedgon.testnet get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id tester.bowtiedgon.testnet
# echo

# echo subscribe with dev account
# echo
# near call tester.bowtiedgon.testnet subscribe '{"creator_address": "bowtiedgon.testnet"}' --account-id tester.bowtiedgon.testnet
# echo

# echo call with dev account which is now subscribed - expect content returned
# echo
# near call tester.bowtiedgon.testnet get_content '{"creator_address": "bowtiedgon.testnet", "date": "28-01-2022"}' --account-id tester.bowtiedgon.testnet
# echo