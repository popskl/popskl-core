use crate::private_proofs::PrivateProof;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, CryptoHash, PanicOnDefault, Promise,
};

mod private_proofs;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Popskl {
    owner: AccountId,
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
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner,
            proofs: LookupMap::new(StorageKeys::Proofs),
            terminated_proofs: LookupMap::new(StorageKeys::TerminatedProofs),
        }
    }

    pub fn withdraw_funds(&self) -> Promise {
        assert!(
            &self.owner == &env::predecessor_account_id(),
            "Only owner can withdraw funds!"
        );
        let storage_costs = (env::storage_usage() as u128) * env::storage_byte_cost();
        let balance = env::account_balance();
        Promise::new(self.owner.clone()).transfer(balance - storage_costs)
    }
}

#[cfg(test)]
mod test_commons {
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Timestamp};

    use super::*;

    pub const ISSUER: &str = "alice.testnet";
    pub const VISITOR: &str = "bob.testnet";
    const OWNER: &str = "carol.testnet";
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

    pub fn popskl() -> Popskl {
        Popskl::new(account(OWNER))
    }

    #[test]
    #[should_panic(expected = "owner")]
    fn should_forbid_non_owner_to_withdraw() {
        // given
        let contract = popskl();
        prepare_context(VISITOR);

        // when
        contract.withdraw_funds();
    }
}