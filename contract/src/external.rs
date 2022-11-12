use near_sdk::{ext_contract, AccountId, PromiseOrValue};

#[ext_contract(this_contract)]
trait Callbacks {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        msg: String,
    ) -> PromiseOrValue<bool>;
}

#[ext_contract(basic_nft)]
trait BasicNFT {
    fn nft_transfer(
        &self,
        receiver_id: AccountId,
        token_id: String,
        appoval_id: Option<u64>,
        memo: Option<String>,
    );
}
