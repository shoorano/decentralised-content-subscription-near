use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, collections::LookupMap, BorshStorageKey};
use serde::Serialize;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    data: LookupMap<String, Profile>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    profile_type: ProfileType,
    content: LookupMap<String, String>,
    subscribers: Vec<String>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Data,
    Content
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, PartialEq, Debug)]
pub enum ProfileType {
    Creator,
    Consumer
}

impl Default for Contract {
    fn default() -> Self {
        let mut data = LookupMap::new(StorageKeys::Data);
        data.insert(
            &"bowtiedgon.testnet".to_owned(),
            &Profile::default()
        );
        Self {
            data
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        let mut content: LookupMap<String, String> = LookupMap::new(
            StorageKeys::Content
        );
        content.insert(
            &"date".to_owned(),
            &"content test".to_owned()
        );
        Self {
            profile_type: ProfileType::Creator,
            content,
            subscribers: vec!["bowtiedgon.testnet".to_owned()]
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_profile(&mut self, account_id: String) -> Option<Profile> {
        self.data.get(&account_id)
    }

    pub fn add_profile(&mut self, account_id: String, profile_type: ProfileType) {
        self.data.insert(
            &account_id,
            &Profile::new(
                profile_type
            )
        );
    }
}

impl Profile {
    pub fn new(profile_type: ProfileType) -> Self {
        let content: LookupMap<String, String> = LookupMap::new(
            StorageKeys::Content
        );
        let subscribers: Vec<String> = Vec::new();
        Self {
            profile_type,
            content,
            subscribers
        }
    }

    pub fn subscribe(&mut self) {
        let account_id = env::signer_account_id();
        self.subscribers.push(account_id);
    }

    pub fn get_content(&self, account_id: String, date: String) -> Option<String> {
        if self.subscribers.contains(&account_id) {
            self.content.get(&date)
        } else {
            env::log(b"Not a subscriber");
            None
        }
    }

    pub fn add_content(&mut self, date: String, content: String) {
        self.content.insert(&date, &content);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bowtiedgon.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn add_profile() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let account_id = "dan.testnet".to_owned();
        let mut contract = Contract::default();
        contract.add_profile(account_id, ProfileType::Consumer);
        let test_profile = Profile::new(
            ProfileType::Consumer
        );
        let profile = match contract.get_profile("dan.testnet".to_owned()) {
            Some(profile) => profile,
            None => panic!()
        };
        assert_eq!(
            test_profile.profile_type,
            profile.profile_type
        );
        assert_eq!(
            test_profile.subscribers,
            profile.subscribers
        );
    }

    #[test]
    fn subscribe() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let mut profile = match contract.get_profile("bowtiedgon.testnet".to_owned()) {
            Some(profile) => profile,
            None => return
        };
        profile.subscribe();
        assert_eq!(
            profile.subscribers[1],
            "bowtiedgon.testnet".to_owned()
        );
    }

    #[test]
    fn get_content_deployer() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let profile = match contract.get_profile("bowtiedgon.testnet".to_owned()) {
            Some(profile) => profile,
            None => return
        };
        assert_eq!(
            Some("content test".to_owned()),
            profile.get_content(
                "bowtiedgon.testnet".to_owned(),
                "date".to_owned()
            )
        );
    }

    #[test]
    fn get_content_non_subscriber() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let profile = match contract.get_profile("bowtiedgon.testnet".to_owned()) {
            Some(profile) => profile,
            None => return
        };
        assert_eq!(
            None,
            profile.get_content(
                "bob_near".to_owned(),
                "date".to_owned()
            )
        );
    }

    #[test]
    fn get_content_subscriber() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let mut profile = match contract.get_profile("bowtiedgon.testnet".to_owned()) {
            Some(profile) => profile,
            None => return
        };
        profile.subscribe();
        assert_eq!(
            Some("content test".to_owned()),
            profile.get_content(
                "bowtiedgon.testnet".to_owned(),
                "date".to_owned()
            )
        );
    }

    #[test]
    fn add_content() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let mut profile = match contract.get_profile("bowtiedgon.testnet".to_owned()) {
            Some(profile) => profile,
            None => return
        };
        profile.add_content("date part 2".to_owned(), "content test part 2".to_owned());
        assert_eq!(
            Some("content test part 2".to_owned()),
            profile.get_content(
                "bowtiedgon.testnet".to_owned(),
                "date part 2".to_owned()
            )
        );
    }
}