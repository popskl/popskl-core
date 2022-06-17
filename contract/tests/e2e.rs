use std::fs;

use anyhow::Result;
use near_sdk::{
    bs58,
    serde_json::{json, Value},
};
use near_units::{parse_near, near};
use sha3::{Digest, Keccak256};
use workspaces::{
    network::Sandbox,
    Account, Worker,
};

const POPSKL_WASM_PATH: &str = "./res/popskl.wasm";

#[tokio::test]
async fn proof_termination_flow() -> Result<()> {
    // given
    let worker = workspaces::sandbox().await?;
    let wasm = fs::read(POPSKL_WASM_PATH)?;
    let owner = create_account(&worker, "carol").await?;
    let popskl = create_account(&worker, "popskl").await?;
    popskl.deploy(&worker, &wasm).await?;
    owner
        .call(&worker, popskl.id(), "new")
        .args_json(json!({ "owner": owner.id() }))?
        .transact()
        .await?;
    let issuer = create_account(&worker, "alice").await?;
    let visitor = create_account(&worker, "bob").await?;
    let proof = "Shipping Location 5|supersecretvalue";
    let args = json!({ "hash": hash(proof) });

    // when
    // call store proof with excessive deposit
    issuer
        .call(&worker, popskl.id(), "store_proof")
        .args_json(args.clone())?
        .deposit(parse_near!("5 N"))
        .transact()
        .await?;

    // check that amount was refunded
    assert!(issuer.view_account(&worker).await?.balance > parse_near!("29 N"));

    // validate proof on chain
    let result: Value = visitor
        .call(&worker, popskl.id(), "validate_proof")
        .args_json(args.clone())?
        .view()
        .await?
        .json()?;
    assert!(result.as_object().unwrap().contains_key("Valid"));

    // terminate proof
    issuer
        .call(&worker, popskl.id(), "terminate_proof")
        .args_json(args.clone())?
        .transact()
        .await?;

    // assert proof is terminated
    let result: Value = visitor
        .call(&worker, popskl.id(), "validate_proof")
        .args_json(args.clone())?
        .view()
        .await?
        .json()?;
    assert!(result.as_object().unwrap().contains_key("Terminated"));

    // assert funds withdrawal
    owner
        .call(&worker, popskl.id(), "withdraw_funds")
        .transact()
        .await?;
    let owner_balance = owner.view_account(&worker).await?.balance;
    println!("Owner balance after withdrawal: {}", near::to_human(owner_balance));
    assert!(owner_balance > parse_near!("55 N"));

    Ok(())
}

fn hash(value: &str) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(value);
    bs58::encode(hasher.finalize()).into_string()
}

async fn create_account(worker: &Worker<Sandbox>, name: &str) -> Result<Account> {
    worker
        .root_account()
        .create_subaccount(worker, name)
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()
}
