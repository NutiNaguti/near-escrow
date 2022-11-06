use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::{collections::UnorderedMap, log, near_bindgen};
use near_sdk::{env, AccountId, Promise};

pub type User = UnorderedMap<String, u128>;

#[derive(BorshSerialize, BorshDeserialize)]
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
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    users: User,
    assets: Vector<Asset>,
    asset_amount: u32,
    user_amount: u32,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            users: UnorderedMap::new(b"m"),
            assets: Vector::new(b"m"),
            asset_amount: 0,
            user_amount: 0,
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    pub fn view_users(&self) -> Vec<(String, u128)> {
        let mut result: Vec<(String, u128)> = vec![];
        for e in self.users.iter() {
            result.push(e);
        }
        result
    }

    pub fn get_balance(&self, user_id: String) -> u128 {
        assert!(!user_id.is_empty(), "user_id is empty");
        let balance = match self.users.get(&user_id) {
            Some(x) => x,
            _ => {
                panic!("\nThis user doesn't exist\n");
            }
        };

        balance
    }

    #[payable]
    pub fn new_user(&mut self) {
        let deposit = env::attached_deposit();
        let sender_id = String::from(env::predecessor_account_id().as_str());

        let _ = match self.users.get(&sender_id) {
            Some(_) => {
                panic!("This user already exist");
            }
            _ => {
                self.users.insert(&sender_id, &deposit);
                self.user_amount += 1;
            }
        };
    }

    #[payable]
    pub fn deposit(&mut self) {
        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Not enougth funds");
        let sender_id = String::from(env::predecessor_account_id().as_str());

        let new_balance = match self.users.get(&sender_id) {
            Some(x) => x + deposit,
            _ => {
                panic!("\nThis user doesn't exist\nYou can call \"new_user\" to register new user");
            }
        };
        log!("Balance updated: {}", new_balance)
    }

    pub fn withdrow_all(&mut self) {
        let sender_id = String::from(env::predecessor_account_id().as_str());
        let balance = match self.users.get(&sender_id) {
            Some(x) => x,
            _ => {
                panic!("\nThis user doesn't exist\nYou can call \"new_user\" to register new user");
            }
        };

        if balance > 0 {
            self.users.insert(&sender_id, &0);
            Promise::new(AccountId::new_unchecked(sender_id)).transfer(balance);
        } else {
            panic!("Not enougth funds");
        }
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
