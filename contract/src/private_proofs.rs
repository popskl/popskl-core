use crate::*;
use near_sdk::{
    env,
    json_types::{Base58CryptoHash, U64},
    serde::Serialize,
    AccountId, Balance, Duration, Promise, Timestamp,
};

type Issuer = AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PrivateProof {
    issuer: Issuer,
    created_at: Timestamp,
    timeout: Option<Duration>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProofView {
    issuer: Issuer,
    #[serde(rename(serialize = "createdAt"))]
    created_at: U64,
    timeout: Option<U64>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ProofStatus {
    Invalid,
    Terminated(ProofView),
    Expired(ProofView),
    Valid(ProofView),
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

    pub fn validate_proof(&self, hash: Base58CryptoHash) -> ProofStatus {
        let crypro_hash = CryptoHash::from(hash);
        self.proofs
            .get(&crypro_hash)
            .map(|proof| {
                proof
                    .timeout
                    .filter(|timeout| (proof.created_at + timeout) < env::block_timestamp())
                    .map(|_| ProofStatus::Expired(ProofView::from(&proof)))
                    .or_else(|| Some(ProofStatus::Valid(ProofView::from(&proof))))
            })
            .flatten()
            .or_else(|| {
                self.terminated_proofs
                    .get(&crypro_hash)
                    .as_ref()
                    .map(ProofView::from)
                    .map(ProofStatus::Terminated)
            })
            .unwrap_or(ProofStatus::Invalid)
    }
}

impl From<&PrivateProof> for ProofView {
    fn from(proof: &PrivateProof) -> Self {
        Self {
            issuer: proof.issuer.clone(),
            created_at: proof.created_at.into(),
            timeout: proof.timeout.map(U64::from),
        }
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
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use near_sdk::testing_env;

    use crate::test_commons::{account, context, prepare_context, CREATED_AT, ISSUER, VISITOR};

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
        let timestamp = popskl.store_proof(Base58CryptoHash::from(hash), None);

        // then
        assert_eq!(timestamp, CREATED_AT);
        assert!(popskl.proofs.contains_key(&hash));
        let stored = popskl.proofs.get(&hash).unwrap();
        assert_proof(stored, test_proof());
    }

    #[test]
    #[should_panic(expected = "already stored")]
    fn should_check_private_proof_uniqueness() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(ISSUER);
        popskl.save_test_proof();

        // when
        popskl.store_proof(Base58CryptoHash::from(hash()), None);
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
        let timestamp = popskl.store_proof(Base58CryptoHash::from(hash), Some(timeout));

        // then
        assert_eq!(timestamp, CREATED_AT);
        assert!(popskl.proofs.contains_key(&hash));
        let stored = popskl.proofs.get(&hash).unwrap();
        assert_proof(stored, expirable_proof((timeout as u64) * 1_000_000_000));
    }

    #[test]
    #[should_panic(expected = "attach")]
    fn should_require_payment_for_new_proof() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(ISSUER);

        // when
        popskl.store_proof(Base58CryptoHash::from(hash()), None);
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
        let terminated = popskl.terminated_proofs.get(&hash).unwrap();
        assert_proof(terminated, test_proof());
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

    #[test]
    fn should_validate_proof() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(VISITOR);
        popskl.save_test_proof();

        // when
        let result = popskl.validate_proof(Base58CryptoHash::from(hash()));

        // then
        match result {
            ProofStatus::Valid(proof) => {
                assert_view(proof, test_proof());
            }
            _ => panic!("Proof should be valid!"),
        }
    }

    #[test]
    fn should_validate_terminated_proof() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(VISITOR);
        popskl.save_terminated_proof();

        // when
        let result = popskl.validate_proof(Base58CryptoHash::from(hash()));

        // then
        match result {
            ProofStatus::Terminated(proof) => {
                assert_view(proof, test_proof());
            }
            _ => panic!("Proof should be terminated!"),
        }
    }

    #[test]
    fn should_validate_non_expired_proof() {
        // given
        let mut popskl = Popskl::new();
        prepare_context(VISITOR);
        let timeout = 60;
        popskl.save_expirable_proof(timeout);

        // when
        let result = popskl.validate_proof(Base58CryptoHash::from(hash()));

        // then
        match result {
            ProofStatus::Valid(proof) => {
                assert_view(proof, expirable_proof(timeout));
            }
            _ => panic!("Proof should be valid!"),
        }
    }

    #[test]
    fn should_validate_expired_proof() {
        // given
        let mut popskl = Popskl::new();
        let timeout = 60;
        popskl.save_expirable_proof(timeout);
        testing_env!(context(VISITOR)
            .block_timestamp(CREATED_AT + timeout + 1)
            .build());

        // when
        let result = popskl.validate_proof(Base58CryptoHash::from(hash()));

        // then
        match result {
            ProofStatus::Expired(proof) => {
                assert_view(proof, expirable_proof(timeout));
            }
            _ => panic!("Proof should be expired!"),
        }
    }

    #[test]
    fn should_validate_invalid_proof() {
        // given
        let popskl = Popskl::new();
        prepare_context(VISITOR);

        // when
        let result = popskl.validate_proof(Base58CryptoHash::from(hash()));

        // then
        match result {
            ProofStatus::Invalid => {}
            _ => panic!("Proof should be invalid!"),
        }
    }

    fn hash() -> CryptoHash {
        CryptoHash::try_from(env::keccak256("12345".as_bytes())).unwrap()
    }

    fn test_proof() -> PrivateProof {
        PrivateProof {
            issuer: account(ISSUER),
            created_at: CREATED_AT,
            timeout: None,
        }
    }

    fn expirable_proof(timeout: u64) -> PrivateProof {
        PrivateProof {
            issuer: account(ISSUER),
            created_at: CREATED_AT,
            timeout: Some(timeout),
        }
    }

    impl Popskl {
        fn save_test_proof(&mut self) {
            self.proofs.insert(&hash(), &test_proof());
        }

        fn save_terminated_proof(&mut self) {
            self.terminated_proofs.insert(&hash(), &test_proof());
        }

        fn save_expirable_proof(&mut self, timeout: u64) {
            self.proofs.insert(&hash(), &expirable_proof(timeout));
        }
    }

    fn assert_proof(actual: PrivateProof, expected: PrivateProof) {
        assert_eq!(actual.issuer, expected.issuer);
        assert_eq!(actual.created_at, expected.created_at);
        assert_eq!(actual.timeout, expected.timeout);
    }

    fn assert_view(actual: ProofView, expected: PrivateProof) {
        assert_eq!(actual.issuer, expected.issuer);
        assert_eq!(actual.created_at, expected.created_at.into());
        assert_eq!(actual.timeout, expected.timeout.map(U64::from));
    }
}
