use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserV1 {
    account_id: AccountId,
    balance: u128,
    asset_ids: UnorderedSet<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedUser {
    V1(UserV1),
}

impl From<VersionedUser> for UserV1 {
    fn from(user: VersionedUser) -> Self {
        match user {
            VersionedUser::V1(u) => u,
        }
    }
}
