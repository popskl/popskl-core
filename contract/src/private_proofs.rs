use crate::*;
use near_sdk::{
    env, json_types::Base58CryptoHash, AccountId, Balance, Duration, Promise, Timestamp,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PrivateProof {
    issuer: AccountId,
    created_at: Timestamp,
    timeout: Option<Duration>,
}

#[near_bindgen]
impl Popskl {
    #[payable]
    pub fn store_proof(&mut self, hash: Base58CryptoHash, timeout: Option<u32>) -> Timestamp {
        let crypto_hash = CryptoHash::from(hash);
        assert!(
            !self.proofs.contains_key(&crypto_hash),
            "Proof is already stored!"
        );

        let issuer = env::predecessor_account_id();
        let created_at = env::block_timestamp();

        assert_payment(|| {
            self.proofs.insert(
                &crypto_hash,
                &PrivateProof {
                    issuer,
                    created_at,
                    timeout: timeout.map(to_nanos),
                },
            );
        });

        created_at
    }

    pub fn terminate_proof(&mut self, hash: Base58CryptoHash) {
        let crypto_hash = CryptoHash::from(hash);
        let proof = self.proofs.remove(&crypto_hash).expect("Proof not found.");
        assert!(
            proof.issuer == env::predecessor_account_id(),
            "Only proof owner is allowed to terminate proof."
        );

        self.terminated_proofs.insert(&crypto_hash, &proof);
    }
}

fn to_nanos(seconds: u32) -> u64 {
    u64::from(seconds) * 1_000_000_000
}

fn assert_payment<F>(storage_update: F)
where
    F: FnOnce(),
{
    let initial_storage = env::storage_usage();

    storage_update();

    let required_cost =
        Balance::from(env::storage_usage() - initial_storage) * env::storage_byte_cost();
    assert!(
        env::attached_deposit() >= required_cost,
        "Requires {} yoctoNEAR attached.",
        required_cost
    );
    let refund = env::attached_deposit() - required_cost;
    if refund > 0 {
        // TODO: add integration test that checks refund
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use near_sdk::testing_env;

    use crate::test_commons::{account, context, prepare_context, ISSUER, TIMESTAMP, VISITOR};

    use super::*;

    #[test]
    fn should_store_private_proof() {
        // given
        let mut popskl = Popskl::new();
        testing_env!(context(ISSUER)
            .attached_deposit(99 * env::storage_byte_cost())
            .build());
        let hash = hash();

        // when
        let timestamp = popskl.store_proof(Base58CryptoHash::from(hash), Option::None);

        // then
        assert_eq!(timestamp, TIMESTAMP);
        assert!(popskl.proofs.contains_key(&hash));
        let stored = popskl.proofs.get(&hash).unwrap();
        assert_eq!(stored.issuer, account(ISSUER));
        assert_eq!(stored.created_at, TIMESTAMP);
        assert_eq!(stored.timeout, Option::None);
    }

    #[test]
    #[should_panic(expected = "already stored")]
    fn should_check_private_proof_uniqueness() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(ISSUER);
        popskl.save_test_proof();

        // when
        popskl.store_proof(Base58CryptoHash::from(hash()), Option::None);
    }

    #[test]
    fn should_save_timeout() {
        // given
        let mut popskl = Popskl::new();
        testing_env!(context(ISSUER)
            .attached_deposit(107 * env::storage_byte_cost())
            .build());
        let hash = hash();
        let timeout = 23;

        // when
        let timestamp = popskl.store_proof(Base58CryptoHash::from(hash), Option::Some(timeout));

        // then
        assert_eq!(timestamp, TIMESTAMP);
        assert!(popskl.proofs.contains_key(&hash));
        let stored = popskl.proofs.get(&hash).unwrap();
        assert_eq!(stored.issuer, account(ISSUER));
        assert_eq!(stored.created_at, TIMESTAMP);
        assert_eq!(
            stored.timeout,
            Option::Some((timeout as u64) * 1_000_000_000)
        );
    }

    #[test]
    #[should_panic(expected = "attach")]
    fn should_require_payment_for_new_proof() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(ISSUER);

        // when
        popskl.store_proof(Base58CryptoHash::from(hash()), Option::None);
    }

    #[test]
    fn should_terminate_proof() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(ISSUER);
        popskl.save_test_proof();
        let hash = hash();

        // when
        popskl.terminate_proof(Base58CryptoHash::from(hash));

        // then
        assert!(!popskl.proofs.contains_key(&hash));
        assert!(popskl.terminated_proofs.contains_key(&hash));
        let expected = test_proof();
        let terminated = popskl.terminated_proofs.get(&hash).unwrap();
        assert_eq!(terminated.issuer, expected.issuer);
        assert_eq!(terminated.created_at, expected.created_at);
        assert_eq!(terminated.timeout, expected.timeout);
    }

    #[test]
    #[should_panic(expected = "proof owner")]
    fn should_check_ownership_before_termination() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(VISITOR);
        popskl.save_test_proof();

        // when
        popskl.terminate_proof(Base58CryptoHash::from(hash()));
    }

    fn hash() -> CryptoHash {
        CryptoHash::try_from(env::keccak256("12345".as_bytes())).unwrap()
    }

    fn test_proof() -> PrivateProof {
        PrivateProof {
            issuer: account(ISSUER),
            created_at: TIMESTAMP,
            timeout: Option::None,
        }
    }

    impl Popskl {
        fn save_test_proof(&mut self) {
            self.proofs.insert(&hash(), &test_proof());
        }
    }
}
