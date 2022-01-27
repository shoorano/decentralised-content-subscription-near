use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env,
    near_bindgen,
    collections::{LookupMap, LookupSet},
    BorshStorageKey,
    AccountId
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    profile_type: ProfileType,
    content: LookupMap<String, String>,
    subscribers: LookupSet<AccountId>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Data,
    Content,
    Subscriber
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub enum ProfileType {
    Creator,
    Consumer
}

impl ProfileType {
    fn new(profile_type: &str) -> Self {
        match profile_type {
            "creator" => Self::Creator,
            "consumer" => Self::Consumer,
            _ => panic!("enter a valid profile type")
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
        let mut subscribers = LookupSet::new(
            StorageKeys::Subscriber
        );
        subscribers.insert(&"bob_near".parse().unwrap());
        Self {
            profile_type: ProfileType::Creator,
            content,
            subscribers
        }
    }
}

impl Profile {
    pub fn new(profile_type: ProfileType) -> Self {
        let content: LookupMap<String, String> = LookupMap::new(
            StorageKeys::Content
        );
        let subscribers = LookupSet::new(
            StorageKeys::Subscriber
        );
        Self {
            profile_type,
            content,
            subscribers
        }
    }

    pub fn subscribe(&mut self) {
        let subscriber_address = env::signer_account_id();
        self.subscribers.insert(&subscriber_address);
    }

    pub fn get_content(&self, date: String) -> Result<String, String> {
        let subscriber_address = env::signer_account_id();
        if self.subscribers.contains(&subscriber_address) {
            match self.content.get(&date) {
                Some(content) => Ok(content),
                None => Err("Could not find content for that date".to_owned())
            }
        } else {
            env::log_str("Not a subscriber");
            Err("Not a subscriber, please subscribe".to_owned())
        }
    }

    pub fn add_content(&mut self, date: String, content: String) {
        self.content.insert(&date, &content);
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    data: LookupMap<AccountId, Profile>
}

impl Default for Contract {
    fn default() -> Self {
        let mut data = LookupMap::new(StorageKeys::Data);
        data.insert(
            &"bob_near".parse().unwrap(),
            &Profile::default()
        );
        Self {
            data
        }
    }
}

#[near_bindgen]
impl Contract {
    #[result_serializer(borsh)]
    pub fn get_profile(&mut self, account_id: AccountId) ->  Option<Profile> {
        self.data.get(&account_id)
    }

    pub fn add_profile(&mut self, account_id: AccountId, profile_type: String) {
        self.data.insert(
            &account_id,
            &Profile::new(
                ProfileType::new(&profile_type)
            )
        );
    }

    pub fn subscribe(&mut self, creator_address: AccountId) {
        let mut profile = match self.get_profile(creator_address) {
            Some(profile) => profile,
            None => return
        };
        profile.subscribe();
    }

    pub fn add_content(&mut self, date: String, content: String) {
        let creator_address = env::signer_account_id();
        let mut profile = match self.get_profile(creator_address) {
            Some(profile) => profile,
            None => return
        };
        profile.add_content(date, content);
    }

    pub fn get_content(&mut self, creator_address: AccountId, date: String) -> String {
        let profile = match self.get_profile(creator_address) {
            Some(profile) => profile,
            None => panic!("this profile does not exist")
        };
        match profile.get_content(date) {
            Ok(content) => content,
            Err(error) => panic!("{}", error)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool, signer_address: AccountId) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(signer_address)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn add_profile() {
        let context = get_context(false, "consumer".parse().unwrap());
        testing_env!(context);
        let account_id = "dan.testnet".parse().unwrap();
        let mut contract = Contract::default();
        contract.add_profile(account_id, "consumer".to_owned());
        let test_profile = Profile::new(
            ProfileType::Consumer
        );
        let profile = match contract.get_profile("dan.testnet".parse().unwrap()) {
            Some(profile) => profile,
            None => panic!()
        };
        assert_eq!(
            test_profile.profile_type,
            profile.profile_type
        );
        assert_eq!(
            test_profile.subscribers.contains(&"consumer".parse().unwrap()),
            profile.subscribers.contains(&"consumer".parse().unwrap())
        );
    }

    #[test]
    fn subscribe() {
        let context = get_context(false, "bob_near".parse().unwrap());
        testing_env!(context);
        let mut contract = Contract::default();
        contract.subscribe("bob_near".parse().unwrap());
        let profile = match contract.get_profile("dan.testnet".parse().unwrap()) {
            Some(profile) => profile,
            None => return
        };
        assert_eq!(
            true,
            profile.subscribers.contains(&"dan.testnet".parse().unwrap())
        );
    }

    #[test]
    fn get_content_deployer() {
        let context = get_context(false, "bob_near".parse().unwrap());
        testing_env!(context);
        let mut contract = Contract::default();
        assert_eq!(
            "content test".to_owned(),
            contract.get_content(
                "bob_near".parse().unwrap(),
                "date".to_owned()
            )
        );
    }

    #[test]
    fn get_content_non_subscriber() {
        let context = get_context(false, "not_bob_near".parse().unwrap());
        testing_env!(context);
        let result = std::panic::catch_unwind(|| {
                let mut contract = Contract::default();
                contract.get_content(
                    "bob_near".parse().unwrap(),
                    "date".to_owned()
                )
            }
        );
        assert!(
            result.is_err()
        );
    }

    #[test]
    fn get_content_subscriber() {
        let context = get_context(false, "bob_near".parse().unwrap());
        testing_env!(context);
        let mut contract = Contract::default();
        contract.subscribe("bob_near".parse().unwrap());
        assert_eq!(
            "content test".to_owned(),
            contract.get_content(
                "bob_near".parse().unwrap(),
                "date".to_owned()
            )
        );
    }

    #[test]
    fn add_content() {
        let context = get_context(false, "bob_near".parse().unwrap());
        testing_env!(context);
        let mut contract = Contract::default();
        contract.add_content("date part 2".to_owned(), "content test part 2".to_owned());
        assert_eq!(
            "content test part 2".to_owned(),
            contract.get_content(
                "bob_near".parse().unwrap(),
                "date part 2".to_owned()
            )
        );
    }
}

