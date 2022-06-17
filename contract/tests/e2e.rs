use std::fs;

use anyhow::Result;
use near_sdk::{
    bs58,
    serde_json::{json, Value},
};
use near_units::parse_near;
use sha3::{Digest, Keccak256};
use workspaces::network::DevAccountDeployer;

const POPSKL_WASM_PATH: &str = "./res/popskl.wasm";

#[tokio::test]
async fn proof_termination_flow() -> Result<()> {
    // given
    let worker = workspaces::sandbox().await?;
    let wasm = fs::read(POPSKL_WASM_PATH)?;
    let contract = worker.dev_deploy(&wasm).await?;
    contract.call(&worker, "new").transact().await?;
    let root = worker.root_account();
    let issuer = root
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let visitor = root
        .create_subaccount(&worker, "bob")
        .transact()
        .await?
        .into_result()?;
    let proof = "Shipping Location 5|supersecretvalue";
    let args = json!({ "hash": hash(proof) });

    // when
    // call store proof with excessive deposit
    issuer
        .call(&worker, contract.id(), "store_proof")
        .args_json(args.clone())?
        .deposit(parse_near!("5 N"))
        .transact()
        .await?;

    // check that amount was refunded
    assert!(issuer.view_account(&worker).await?.balance > parse_near!("29 N"));

    // validate proof on chain
    let result: Value = visitor
        .call(&worker, contract.id(), "validate_proof")
        .args_json(args.clone())?
        .view()
        .await?
        .json()?;
    assert!(result.as_object().unwrap().contains_key("Valid"));

    // terminate proof
    issuer
        .call(&worker, contract.id(), "terminate_proof")
        .args_json(args.clone())?
        .transact()
        .await?;

    // assert proof is terminated
    let result: Value = visitor
        .call(&worker, contract.id(), "validate_proof")
        .args_json(args.clone())?
        .view()
        .await?
        .json()?;
    assert!(result.as_object().unwrap().contains_key("Terminated"));

    Ok(())
}

fn hash(value: &str) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(value);
    bs58::encode(hasher.finalize()).into_string()
}
