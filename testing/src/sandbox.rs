mod utils;
use utils::utils::*;

const DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH: &str = "contracts/res/decentralised_content_subscription_near.wasm";
fn main() {
    build_contract();
    get_wasm(DECENTRALISED_CONTENT_SUBSCRIPTION_NEAR_WASM_FILEPATH);
    remove_near_credentials();
}