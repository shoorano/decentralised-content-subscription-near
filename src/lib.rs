use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    collections::{LookupMap, LookupSet},
    BorshStorageKey,
    AccountId,
    json_types::U128,
    Promise,
    near_bindgen,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    profile_type: ProfileType,
    content: LookupMap<String, String>,
    subscribers: LookupSet<AccountId>,
    cost: U128
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
        let subscribers = LookupSet::new(
            StorageKeys::Subscriber
        );
        Self {
            profile_type: ProfileType::Creator,
            content,
            subscribers,
            cost: U128::from(10u128.pow(25))
        }
    }
}

impl Profile {
    pub fn new(profile_type: ProfileType, cost: U128) -> Self {
        let content: LookupMap<String, String> = LookupMap::new(
            StorageKeys::Content
        );
        let subscribers = LookupSet::new(
            StorageKeys::Subscriber
        );
        Self {
            profile_type,
            content,
            subscribers,
            cost
        }
    }

    pub fn subscribe(&mut self) {
        let subscriber_address = env::signer_account_id();
        self.subscribers.insert(&subscriber_address);
    }

    pub fn get_content(&self, date: String, is_owner: bool) -> Result<String, String> {
        if is_owner {
            match self.content.get(&date) {
                Some(content) => Ok(content),
                None => Err("Could not find content for that date".to_owned())
            }
        } else {
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
    }

    pub fn add_content(&mut self, date: String, content: String) {
        match self.profile_type {
            ProfileType::Creator => self.content.insert(&date, &content),
            ProfileType::Consumer => panic!(
                "{}",
                "Please create a creator profile to add content".to_owned()
            )
        };
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    data: LookupMap<AccountId, Profile>,
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
    pub fn get_profile(&mut self, account_id: &AccountId) ->  Option<Profile> {
        self.data.get(&account_id)
    }

    pub fn add_profile(&mut self, account_id: AccountId, profile_type: String, cost: String) {
        let cost = U128::from(cost.parse::<u128>().unwrap());
        self.data.insert(
            &account_id,
            &Profile::new(
                ProfileType::new(&profile_type),
                cost
            )
        );
    }

    pub fn subscribe(&mut self, creator_address: AccountId) {
        let mut profile = match self.get_profile(&creator_address) {
            Some(profile) => profile,
            None => return
        };
        let amount =profile.cost;
        assert!(!profile.subscribers.contains(&env::signer_account_id()));
        Promise::new(creator_address).transfer(amount.0);
        profile.subscribe();
    }

    pub fn add_content(&mut self, date: String, content: String) {
        let creator_address = env::signer_account_id();
        let mut profile = match self.get_profile(&creator_address) {
            Some(profile) => profile,
            None => return
        };
        profile.add_content(date, content);
    }

    pub fn get_content(&mut self, creator_address: AccountId, date: String) -> String {
        let profile = match self.get_profile(&creator_address) {
            Some(profile) => profile,
            None => panic!("this profile does not exist")
        };
        let is_owner = env::signer_account_id() == creator_address;
        match profile.get_content(date, is_owner) {
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

    fn get_context(is_view: bool, signer_address: AccountId, signer_balance: u128) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id(signer_address)
            .account_balance(signer_balance)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn add_profile() {
        let context = get_context(
            false,
            "consumer".parse().unwrap(),
            10u128.pow(20)
        );
        testing_env!(context);
        let account_id = "dan.testnet".parse().unwrap();
        let mut contract = Contract::default();
        contract.add_profile(account_id, "consumer".to_owned(), "10000".to_owned());
        let test_profile = Profile::new(
            ProfileType::Consumer,
            U128::from(10u128.pow(20))
        );
        let profile = match contract.get_profile(&"dan.testnet".parse().unwrap()) {
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
        let context = get_context(
            false,
            "dan.testnet".parse().unwrap(),
            10u128.pow(25)
        );
        testing_env!(context);
        let mut contract = Contract::default();
        contract.subscribe("bob_near".parse().unwrap());
        let profile = match contract.get_profile(&"bob_near".parse().unwrap()) {
            Some(profile) => profile,
            None => return
        };
        assert_eq!(
            true,
            profile.subscribers.contains(&"dan.testnet".parse().unwrap())
        );
    }

    #[test]
    fn subscribe_low_balance() {
        let context = get_context(
            false,
            "dan.testnet".parse().unwrap(),
            10u128.pow(24)
        );
        testing_env!(context);
        let result = std::panic::catch_unwind(|| 
            {
                let mut contract = Contract::default();
                contract.subscribe("bob_near".parse().unwrap());
            }
        );
        assert!(
            result.is_err()
        );
    }

    #[test]
    fn get_content_non_subscriber() {
        let context = get_context(
            false,
            "not_bob_near".parse().unwrap(),
            10u128.pow(25)
        );
        testing_env!(context);
        let result = std::panic::catch_unwind(||
            {
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
        let context = get_context(
            false,
            "dan_testnet".parse().unwrap(),
            10u128.pow(25)
        );
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
    fn add_content_creator() {
        let context = get_context(
            false,
            "bob_near".parse().unwrap(),
            10u128.pow(25)
        );
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

    #[test]
    fn add_content_consumer() {
        let context = get_context(
            false,
            "consumer".parse().unwrap(),
            10u128.pow(25)
        );
        testing_env!(context);
        let result = std::panic::catch_unwind(|| 
            {
                let mut contract = Contract::default();
                contract.add_profile(
                    "consumer".parse().unwrap(),
                    "consumer".to_owned(),
                    "100000000".to_owned()
                );
                contract.add_content(
                    "date part 2".to_owned(),
                    "content test part 2".to_owned()
                );
            }
        );
        assert!(
            result.is_err()
        );
    }
}

#[cfg(test)]
mod test {
    use workspaces::prelude::*;

    #[tokio::test]
    async fn test_deploy_and_view() -> anyhow::Result<()> {
        let worker = workspaces::testnet();
        
        let contract = worker.dev_deploy(include_bytes!(
            "../target/wasm32-unknown-unknown/release/decentralised_patreon.wasm")
            .to_vec()
        )
            .await
            .expect("could not dev-deploy contract");

        let result: String = contract.view(
            &worker,
            "add_profile",
            serde_json::json!({
                "account_id": contract.id(),
                "profile_type": "creator",
                "cost": "10000000000000000000000"
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
        
        assert_eq!(result, "1");
        Ok(())
    }
}