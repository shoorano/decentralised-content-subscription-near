mod data_structures;
use data_structures::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    collections::{LookupMap},
    AccountId,
    json_types::U128,
    Promise,
    near_bindgen
};

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

    pub fn add_profile(&mut self, account_id: AccountId, profile_type: String, cost: String, payment_interval: String) {
        let cost_in_yocto_near = U128::from(
            cost.parse::<u128>().unwrap() * 1_000_000_000_000_000_000_000_000
        );
        let payment_interval = payment_interval.parse::<i32>().unwrap();
        self.data.insert(
            &account_id,
            &Profile::new(
                ProfileType::new(&profile_type),
                cost_in_yocto_near,
                payment_interval
            )
        );
    }

    pub fn subscribe(&mut self, creator_address: AccountId) {
        let mut profile = match self.get_profile(&creator_address) {
            Some(profile) => profile,
            None => return
        };
        let amount = match profile.costs.get(&"cost".to_owned()) {
            Some(cost) => cost,
            None => panic!("could not access cost")
        };
        if let Some(content_count) = profile.content_count.get(&"content_count".to_owned()) {
            match profile.subscribers.get(&env::signer_account_id()) {
                Some(count) => {
                    if content_count > count + profile.payment_interval {
                        Promise::new(creator_address).transfer(amount.0);
                        profile.subscribe();
                    } else {
                        env::log_str("User has content left on current subscription");
                        panic!("User has content left on current subscription");
                    }
                },
                None => {
                    Promise::new(creator_address).transfer(amount.0);
                    profile.subscribe();
                }
            }
        }
    }

    pub fn add_content(&mut self, date: String, content: String) {
        let creator_address = env::signer_account_id();
        let mut profile = match self.get_profile(&creator_address) {
            Some(profile) => profile,
            None => return
        };
        if let Some(current_content_count) = profile.content_count.get(&"content_count".to_owned()) {
            profile.content_count.insert(&"content_count".to_owned(), &(current_content_count + 1));
        }
        profile.add_content(date, content);
    }

    pub fn get_content(&mut self, creator_address: AccountId, date: String) -> String {
        let profile = match self.get_profile(&creator_address) {
            Some(profile) => profile,
            None => panic!("this profile does not exist")
        };
        let is_owner = env::signer_account_id() == creator_address;
        match profile.get_content(
            date,
            is_owner
        ) {
            Ok(content) => content,
            Err(error) => panic!("{}", error)
        }
    }

    pub fn get_cost(mut self) -> String {
        let account_id = env::signer_account_id();
        let profile = match self.get_profile(&account_id) {
            Some(profile) => profile,
            None => panic!("this profile does not exist")
        };
        let cost = match profile.costs.get(&"cost".to_owned()) {
            Some(cost) => cost,
            None => panic!("could not access cost")
        };
        format!("{}", cost.0 / 1_000_000_000_000_000_000_000_000)
    }

    pub fn update_cost(&mut self, cost: String) {
        let account_id = env::signer_account_id();
        let profile = match self.get_profile(&account_id) {
            Some(profile) => profile,
            None => panic!("this profile does not exist")
        };
        let cost_in_yocto_near = U128::from(
            cost.parse::<u128>().unwrap() * 1_000_000_000_000_000_000_000_000
        );
        profile.update_cost(cost_in_yocto_near);
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
    fn test_add_profile() {
        let context = get_context(
            false,
            "consumer".parse().unwrap(),
            10u128.pow(20)
        );
        testing_env!(context);
        let account_id = "dan.testnet".parse().unwrap();
        let mut contract = Contract::default();
        contract.add_profile(
            account_id,
            "consumer".to_owned(),
            "1".to_owned(),
            "4".to_owned()
        );
        let test_profile = Profile::new(
            ProfileType::Consumer,
            U128::from(10u128.pow(20)),
            4
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
            test_profile.subscribers.get(&"consumer".parse().unwrap()),
            profile.subscribers.get(&"consumer".parse().unwrap())
        );
    }

    #[test]
    fn test_subscribe() {
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
            Some(1),
            profile.subscribers.get(&"dan.testnet".parse().unwrap())
        );
    }

    #[test]
    fn test_subscribe_low_balance() {
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
    fn test_get_content_non_subscriber() {
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
    fn test_get_content_subscriber() {
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
    fn test_add_content_creator() {
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
    fn test_add_content_consumer() {
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
                    "1".to_owned(),
                    "4".to_owned()
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

    #[test]
    fn test_update_cost() {
        let context = get_context(
            false,
            "creator".parse().unwrap(),
            10u128.pow(25)
        );
        testing_env!(context);

        let mut contract = Contract::default();
        contract.add_profile(
            "creator".parse().unwrap(),
            "creator".to_owned(),
            "3".to_owned(),
            "4".to_owned()
        );
        contract.update_cost(
            "2".to_owned(),
        );
        assert_eq!(
            contract.get_cost(),
            "2".to_owned()
        );
    }
}