# PoPskl Contract

Blockchain part of the project.
Provides storage and operations on location proofs.

Generally, there are two types of proofs: private and public ones. 
Visitors can obtain and keep location proof for themselves or submit it for public view.

For more info consult Rust docs `cargo doc --open` or [near-cli usage section](#usage).

## Build & Deploy

To encapsulate all scripts in one place `Makefile` is added.

Requires:
* `rust` & `cargo`
* `wasm32-unknown-unknown` toolchain target
* `near` executable

```bash
# removes generated files
make clean

# produce release build
make build

# run cargo tests
make unit-test
make e2e-test

# deploy to popskl.vchernetskyi.testnet
make dev-deploy owner=vchernetskyi.testnet
```

## CLI Usage

Usage examples assume that you've `dev-deploy`ed popskl contract.

### Prepare

Set accounts:
```bash
OWNER=vchernetskyi.testnet
ISSUER=$OWNER
VISITOR=$OWNER
CONTRACT=popskl1.$OWNER
```
**Note:** For test purposes we are using the same account for owner, issuer and visitor.
In real life those are (probably?) 3 different persons.

Build utility scripts:
```bash
make build-scripts
```

Generate decoded proof:
```bash
LOCATION="My Custom Location"
SECRET=$(node scripts/out/popskl-helper.js secret)
HASH=$(node scripts/out/popskl-helper.js hash "$LOCATION" "$SECRET")
```

### Interact

Issue new location proof:
```bash
# without timeout
near call $CONTRACT store_proof \
    --args "{\"hash\": \"$HASH\"}" \
    --accountId $ISSUER \
    --depositYocto 3060000000000000000000

# with timeout in seconds
near call $CONTRACT store_proof \
    --args "{\"hash\": \"$HASH\", \"timeout\": 60}" \
    --accountId $ISSUER \
    --depositYocto 3140000000000000000000
```
**Note:** `store_proof` charges for proof storage and additionally 0.002 NEAR.
Any excessive attached deposit is refunded.

Terminate private location proof:
```bash
near call $CONTRACT terminate_proof \
    --args "{\"hash\": \"$HASH\"}" \
    --accountId $ISSUER
```

Validate location proof:
```bash
near view $CONTRACT validate_proof --args "{\"hash\": \"$HASH\"}"
```

Withdraw funds:
```bash
near call $CONTRACT withdraw_funds --accountId $OWNER
```
