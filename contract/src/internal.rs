use near_sdk::{env, CryptoHash};

use crate::Version;

pub(crate) fn hash_str(str: &str) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(str.as_bytes()));
    hash
}

impl Version {
    pub fn inc(&mut self) {
        self.2 += 1;
    }
}
