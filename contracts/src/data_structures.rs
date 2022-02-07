use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    collections::LookupMap,
    BorshStorageKey,
    AccountId,
    json_types::U128,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    pub profile_type: ProfileType,
    pub content: LookupMap<String, String>,
    pub subscribers: LookupMap<AccountId, i32>,
    pub costs: LookupMap<String, U128>,
    pub content_count: i32,
    pub payment_interval: i32
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Data,
    Content,
    Subscribers,
    Cost
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub enum ProfileType {
    Creator,
    Consumer
}

impl ProfileType {
    pub fn new(profile_type: &str) -> Self {
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
        let subscribers = LookupMap::new(
            StorageKeys::Subscribers
        );
        let mut costs: LookupMap<String, U128> = LookupMap::new(
            StorageKeys::Cost
        );
        costs.insert(&"cost".to_owned(), &U128::from(10u128.pow(25)));
        Self {
            profile_type: ProfileType::Creator,
            content,
            subscribers,
            costs,
            content_count: 1,
            payment_interval: 4
        }
    }
}

impl Profile {
    pub fn new(profile_type: ProfileType, cost: U128, payment_interval: i32) -> Self {
        let content: LookupMap<String, String> = LookupMap::new(
            StorageKeys::Content
        );
        let subscribers = LookupMap::new(
            StorageKeys::Subscribers
        );
        let mut costs = LookupMap::<String, U128>::new(
            StorageKeys::Cost
        );
        costs.insert(&"cost".to_owned(), &cost);
        Self {
            profile_type,
            content,
            subscribers,
            costs,
            content_count: 0,
            payment_interval
        }
    }

    pub fn subscribe(&mut self) {
        let subscriber_address = env::signer_account_id();
        self.subscribers.insert(&subscriber_address, &self.content_count);
    }

    pub fn get_content(&self, date: String, is_owner: bool) -> Result<String, String> {
        if is_owner {
            match self.content.get(&date) {
                Some(content) => Ok(content),
                None => Err("Could not find content for that date".to_owned())
            }
        } else {
            let subscriber_address = env::signer_account_id();
            match self.subscribers.get(&subscriber_address) {
                Some(count) => {
                    if self.content_count <= count + self.payment_interval {
                        match self.content.get(&date) {
                            Some(content) => Ok(content),
                            None => Err("Could not find content for that date".to_owned())
                        }
                    } else {
                        env::log_str("Please top up as current subscription has ended");
                        Err("Please top up as current subscription has ended".to_owned())
                    }
                },
                None => {
                    env::log_str("Not a subscriber");
                    Err("Not a subscriber, please subscribe".to_owned())
                }
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

    pub fn update_cost(mut self, cost: U128) {
        self.costs.insert(&"cost".to_owned(), &cost);
    }
}