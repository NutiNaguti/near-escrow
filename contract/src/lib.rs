use external::basic_nft;
use internal::hash_str;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::{collections::UnorderedMap, collections::Vector, log, near_bindgen};
use near_sdk::{env, require, AccountId, Gas, PanicOnDefault, Promise, PromiseError};

mod external;
mod internal;
mod migrate;

const TGAS: u64 = 1_000_000_000_000;

// hardcoded nft contract address
const NFT_ACCOUNT_ID: &str = "dev-1667910219580-96853394592542";

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Version(u8, u8, u8);

#[derive(BorshSerialize, BorshDeserialize)]
pub struct User {
    account_id: AccountId,
    balance: u128,
    asset_ids: UnorderedSet<String>,
}

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Asset {
    price: u128,
    init_time: u64,
    last_time: u64,
    last_owner: AccountId,
    last_user: AccountId,
    active: bool,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    user_index: UnorderedMap<AccountId, u64>,
    users: Vector<User>,
    assets: UnorderedMap<String, Asset>,
    asset_amount: u64,
    user_amount: u64,
    escrow_ver: Version,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            user_index: UnorderedMap::new(hash_str("indexes").try_to_vec().unwrap()),
            users: Vector::new(hash_str("users").try_to_vec().unwrap()),
            assets: UnorderedMap::new(hash_str("assets").try_to_vec().unwrap()),
            asset_amount: 0,
            user_amount: 0,
            escrow_ver: Version(0, 0, 1),
        }
    }

    pub fn view_users(&self) -> Vec<AccountId> {
        let mut users = vec![];
        for e in self.users.iter() {
            users.push(e.account_id);
        }
        users
    }

    pub fn view_user(&self) -> (AccountId, Vec<String>, u128) {
        let user = match self.user_index.get(&env::predecessor_account_id()) {
            Some(i) => self.users.get(i).unwrap(),
            _ => {
                panic!("User not found");
            }
        };

        (user.account_id, user.asset_ids.to_vec(), user.balance)
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
                    asset_ids: UnorderedSet::new(
                        hash_str(&format!("asset_index_{}", self.escrow_ver.2)).to_vec(),
                    ),
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

    pub fn reset_state(&mut self) {
        self.escrow_ver.inc();
        self.user_index = UnorderedMap::new(
            hash_str(&format!("indexes_{}", self.escrow_ver.2))
                .try_to_vec()
                .unwrap(),
        );
        self.users = Vector::new(
            hash_str(&format!("users_{}", self.escrow_ver.2))
                .try_to_vec()
                .unwrap(),
        );
        self.assets = UnorderedMap::new(
            hash_str(&format!("assets_{}", self.escrow_ver.2))
                .try_to_vec()
                .unwrap(),
        );
        self.asset_amount = 0;
        self.user_amount = 0;
    }

    pub fn get_current_ver(&self) -> (u8, u8, u8) {
        (self.escrow_ver.0, self.escrow_ver.1, self.escrow_ver.2)
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
