use external::basic_nft;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::{collections::UnorderedMap, collections::UnorderedSet, log, near_bindgen};
use near_sdk::{
    env, require, AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseError,
};

mod external;
mod internal;
mod versioned_asset;
mod versioned_user;

const TGAS: u64 = 1_000_000_000_000;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    balance: u128,
    asset_ids: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Asset {
    price: u128,
    init_time: u64,
    last_time: u64,
    last_owner: AccountId,
    last_user: AccountId,
    active: bool,
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    Users,
    Assets,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct EscrowContract {
    nft_account_id: AccountId,
    users: UnorderedMap<AccountId, User>,
    assets: UnorderedMap<String, Asset>,
    asset_amount: u16,
    user_amount: u16,
}

#[near_bindgen]
impl EscrowContract {
    #[init]
    #[private]
    pub fn new(nft_account_id: AccountId) -> Self {
        Self {
            nft_account_id,
            users: UnorderedMap::new(StorageKey::Users),
            assets: UnorderedMap::new(StorageKey::Assets),
            asset_amount: 0,
            user_amount: 0,
        }
    }

    pub fn view_users(&self) -> Vec<(AccountId, User)> {
        self.users.to_vec()
    }

    pub fn view_user(&self) -> User {
        match self.users.get(&env::predecessor_account_id()) {
            Some(user) => return user,
            _ => {
                panic!("User not found");
            }
        }
    }

    pub fn get_balance(&self) -> u128 {
        match self.users.get(&env::predecessor_account_id()) {
            Some(user) => {
                return user.balance;
            }
            _ => {
                panic!("\nThis user doesn't exist\n");
            }
        };
    }

    #[payable]
    pub fn new_user(&mut self) {
        let deposit = env::attached_deposit();
        let sender_id = env::predecessor_account_id();

        match self.users.get(&sender_id) {
            Some(_) => {
                panic!("This user already exist");
            }
            _ => {
                self.user_amount += 1;
                self.users.insert(
                    &sender_id,
                    &User {
                        balance: deposit,
                        asset_ids: vec![],
                    },
                );
            }
        };
    }

    #[payable]
    pub fn deposit(&mut self) {
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Not enougth funds");
        let sender_id = env::predecessor_account_id();

        match self.users.get(&sender_id) {
            Some(user) => {
                let new_balance = user.balance + deposit;
                self.users.insert(
                    &sender_id,
                    &User {
                        balance: new_balance,
                        asset_ids: user.asset_ids,
                    },
                )
            }
            _ => {
                panic!("\nThis user doesn't exist\nYou can call \"new_user\" to register new user");
            }
        };
    }

    pub fn withdrow_all(&mut self) -> Promise {
        let sender_id = env::predecessor_account_id();
        match self.users.get(&sender_id) {
            Some(user) => {
                let balance = user.balance;
                if balance > 0 {
                    self.users.insert(
                        &sender_id,
                        &User {
                            balance: 0,
                            asset_ids: user.asset_ids,
                        },
                    );
                    return Promise::new(sender_id).transfer(balance);
                } else {
                    panic!("Not enougth funds");
                }
            }
            _ => {
                panic!("\nThis user doesn't exist\nYou can call \"new_user\" to register new user");
            }
        };
    }

    pub fn view_assets(&self) -> Vec<(String, Asset)> {
        self.assets.to_vec()
    }

    pub fn view_asset(&self, token_id: String) -> Asset {
        match self.assets.get(&token_id) {
            Some(asset) => return asset,
            _ => panic!("Asset not found"),
        }
    }

    #[payable]
    pub fn place_new_asset(
        &mut self,
        token_id: String,
        price: u128,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        match self.assets.get(&token_id) {
            Some(_) => panic!("Asset already placed"),
            _ => self.query_transfer(
                env::current_account_id(),
                token_id.clone(),
                approval_id,
                memo,
            ),
        };

        self.assets.insert(
            &token_id,
            &Asset {
                price,
                init_time: env::block_timestamp(),
                last_time: env::block_timestamp(),
                last_user: env::predecessor_account_id(),
                last_owner: env::predecessor_account_id(),
                active: true,
            },
        );
        self.asset_amount += 1;
    }

    #[payable]
    pub fn buy_asset(&mut self, token_id: String) {
        let mut asset = match self.assets.get(&token_id) {
            Some(asset) => {
                require!(asset.active, "Asset already was sold");
                require!(env::attached_deposit() >= asset.price, "Not enougth funds");
                asset
            }
            _ => panic!("Asset doesn't placed"),
        };

        self.query_transfer(env::predecessor_account_id(), token_id.clone(), None, None);
        Promise::new(asset.last_owner.clone()).transfer(asset.price);
        asset.last_user = env::predecessor_account_id();
        asset.last_time = env::block_timestamp();
        asset.active = false;
        self.assets.insert(&token_id, &asset);
    }

    #[payable]
    fn query_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Promise {
        let promise = basic_nft::ext(self.nft_account_id.clone())
            .with_static_gas(Gas(5 * TGAS))
            .with_attached_deposit(env::attached_deposit())
            .nft_transfer(receiver_id, token_id, approval_id, memo);

        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(5 * TGAS))
                .query_transfer_callback(),
        )
    }

    #[private]
    pub fn query_transfer_callback(
        &self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> bool {
        let mut result = false;

        if !call_result.is_err() {
            result = true;
            return result;
        }

        log!("{:?}", call_result.err().unwrap());
        result
    }

    pub fn reset_state(&mut self) {
        self.users = UnorderedMap::new(StorageKey::Users);
        self.assets = UnorderedMap::new(StorageKey::Assets);
        self.asset_amount = 0;
        self.user_amount = 0;
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env, ONE_NEAR};

    use super::*;

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn test_new_user() {
        let predecessor: AccountId = "foo".parse().unwrap();
        let context = get_context(predecessor.clone());
        testing_env!(context.build());
        let mut contract = EscrowContract::new("dev-1667910219580-96853394592542".parse().unwrap());
        contract.new_user();
        assert_eq!(contract.users.get(&predecessor).unwrap().balance, 0);
        assert_eq!(
            contract.users.get(&predecessor).unwrap().asset_ids,
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_deposit() {
        let predecessor: AccountId = "foo".parse().unwrap();
        let mut context = get_context(predecessor.clone());
        testing_env!(context.attached_deposit(ONE_NEAR).build());
        let mut contract = EscrowContract::new("dev-1667910219580-96853394592542".parse().unwrap());
        contract.new_user();
        println!(
            "balance: {}",
            contract.users.get(&predecessor).unwrap().balance
        );
        contract.deposit();
        println!(
            "balance: {}",
            contract.users.get(&predecessor).unwrap().balance
        );

        contract.deposit();
        println!(
            "balance: {}",
            contract.users.get(&predecessor).unwrap().balance
        );

        assert_eq!(
            contract.users.get(&predecessor).unwrap().balance,
            ONE_NEAR * 3
        );
    }

    #[test]
    fn test_view_users() {
        let predecessor: AccountId = "foo".parse().unwrap();
        let mut context = get_context(predecessor.clone());
        testing_env!(context.attached_deposit(ONE_NEAR).build());
        let mut contract = EscrowContract::new("dev-1667910219580-96853394592542".parse().unwrap());
        contract.new_user();
        let users = contract.view_users();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].0, predecessor);
        assert_eq!(users[0].1.balance, ONE_NEAR);
        assert_eq!(users[0].1.asset_ids, Vec::<String>::new());
    }

    #[test]
    fn test_view_user() {
        let predecessor: AccountId = "foo".parse().unwrap();
        let mut context = get_context(predecessor.clone());
        testing_env!(context.attached_deposit(ONE_NEAR).build());
        let mut contract = EscrowContract::new("dev-1667910219580-96853394592542".parse().unwrap());
        contract.new_user();
        let user = contract.view_user();
        assert_eq!(user.balance, ONE_NEAR);
        assert_eq!(user.asset_ids, Vec::<String>::new());
    }

    #[test]
    fn test_withdrow_all() {
        let predecessor: AccountId = "foo".parse().unwrap();
        let mut context = get_context(predecessor.clone());
        testing_env!(context.attached_deposit(ONE_NEAR).build());
        let mut contract = EscrowContract::new("dev-1667910219580-96853394592542".parse().unwrap());
        contract.new_user();
        contract.withdrow_all();
        let user = contract.view_user();
        assert_eq!(user.balance, 0);
    }
}
