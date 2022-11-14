// use crate::*;

// #[near_bindgen]
// impl Contract {
//     #[private]
//     #[init(ignore_state)]
//     pub fn migrate() -> Self {
//         #[derive(BorshDeserialize)]
//         struct OldUser {
//             account_id: AccountId,
//             balance: u128,
//         }

//         #[derive(BorshDeserialize)]
//         struct OldState {
//             user_index: UnorderedMap<AccountId, u64>,
//             users: Vector<OldUser>,
//             assets: Vector<Asset>,
//             asset_amount: u64,
//             user_amount: u64,
//         }
//         let old_state: OldState = env::state_read().expect("failed");

//         let mut new_users: Vector<crate::User> =
//             Vector::new(hash_str("users").try_to_vec().unwrap());

//         for e in old_state.users.iter() {
//             new_users.push(&crate::User {
//                 account_id: e.account_id.clone(),
//                 balance: e.balance,
//                 asset_amount: 0,
//                 asset_index: UnorderedMap::new(
//                     hash_str(e.account_id.as_str()).try_to_vec().unwrap(),
//                 ),
//             })
//         }

//         Self {
//             user_index: old_state.user_index,
//             users: new_users,
//             assets: old_state.assets,
//             asset_amount: old_state.asset_amount,
//             user_amount: old_state.user_amount,
//             escrow_ver: Version(0, 0, 1),
//         }
//     }
// }
