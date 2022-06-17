use crate::private_proofs::PrivateProof;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{near_bindgen, BorshStorageKey, CryptoHash, PanicOnDefault};

mod private_proofs;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Popskl {
    proofs: LookupMap<CryptoHash, PrivateProof>,
    terminated_proofs: LookupMap<CryptoHash, PrivateProof>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    Proofs,
    TerminatedProofs,
}

#[near_bindgen]
impl Popskl {
    #[init]
    pub fn new() -> Self {
        Self {
            proofs: LookupMap::new(StorageKeys::Proofs),
            terminated_proofs: LookupMap::new(StorageKeys::TerminatedProofs),
        }
    }
}

#[cfg(test)]
mod test_commons {
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Timestamp};

    pub const ISSUER: &str = "alice.testnet";
    pub const VISITOR: &str = "bob.testnet";
    pub const CREATED_AT: Timestamp = 123456;

    pub fn prepare_context(predecessor: &str) {
        let context = context(predecessor).build();
        testing_env!(context);
    }

    pub fn context(predecessor: &str) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();

        builder
            .predecessor_account_id(account(predecessor))
            .block_timestamp(CREATED_AT);

        builder
    }

    pub fn account(account: &str) -> AccountId {
        AccountId::new_unchecked(account.to_string())
    }
}
