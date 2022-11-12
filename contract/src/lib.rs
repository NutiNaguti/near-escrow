use external::basic_nft;
use internal::hash_str;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{collections::UnorderedMap, log, near_bindgen};
use near_sdk::{env, AccountId, Gas, PanicOnDefault, Promise, PromiseError};

mod external;
mod internal;

const TGAS: u64 = 1_000_000_000_000;
const NFT_ACCOUNT_ID: &str = "dev-1667910219580-96853394592542";

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault, Clone)]
pub struct User {
    account_id: AccountId,
    balance: u128,
}

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Asset {
    token: UnorderedMap<String, u64>,
    init_time: u64,
    last_time: u64,
    init_price: u128,
    last_user: User,
    current_price: u128,
    last_owner: User,
    active: bool,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    user_index: UnorderedMap<AccountId, u64>,
    users: Vector<User>,
    assets: Vector<Asset>,
    asset_amount: u64,
    user_amount: u64,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            user_index: UnorderedMap::new(hash_str("indexes").try_to_vec().unwrap()),
            users: Vector::new(hash_str("users").try_to_vec().unwrap()),
            assets: Vector::new(hash_str("assets").try_to_vec().unwrap()),
            asset_amount: 0,
            user_amount: 0,
        }
    }

    pub fn view_users(&self) -> Vec<(AccountId, u128)> {
        let mut users = vec![];
        for e in self.users.iter() {
            users.push((e.account_id, e.balance));
        }
        users
    }

    pub fn get_balance(&self, user_id: AccountId) -> u128 {
        assert!(!user_id.as_str().is_empty(), "user_id is empty");
        match self.user_index.get(&user_id) {
            Some(i) => {
                return self.users.get(i).unwrap().balance;
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

        match self.user_index.get(&sender_id) {
            Some(_) => {
                panic!("This user already exist");
            }
            _ => {
                self.user_index.insert(&sender_id, &self.user_amount);
                self.user_amount += 1;
                self.users.push(&User {
                    account_id: sender_id,
                    balance: deposit,
                });
            }
        };
    }

    #[payable]
    pub fn deposit(&mut self) {
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Not enougth funds");
        let sender_id = env::predecessor_account_id();

        match self.user_index.get(&sender_id) {
            Some(i) => {
                self.users.get(i).unwrap().balance += deposit;
            }
            _ => {
                panic!("\nThis user doesn't exist\nYou can call \"new_user\" to register new user");
            }
        };
    }

    pub fn withdrow_all(&mut self) -> Promise {
        let sender_id = env::predecessor_account_id();
        match self.user_index.get(&sender_id) {
            Some(i) => {
                let balance = self.users.get(i).unwrap().balance;
                if balance > 0 {
                    self.users.get(i).unwrap().balance = 0;
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

    #[payable]
    pub fn query_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Promise {
        let promise = basic_nft::ext(AccountId::new_unchecked(String::from(NFT_ACCOUNT_ID)))
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
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
}
