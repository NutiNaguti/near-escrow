use near_sdk::{ext_contract, AccountId};

#[ext_contract(this_contract)]
trait Callbacks {
    fn query_greeting_callback(&mut self) -> String;
    fn query_nft_transfer_callback(&mut self) -> String;
}

#[ext_contract(basic_nft)]
trait BasicNFT {
    fn nft_transfer(&self, receiver_id: AccountId, token_id: String, memo: Option<String>);
}
