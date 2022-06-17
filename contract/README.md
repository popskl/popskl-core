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
make test

# deploy to testnet
make dev-deploy account=vchernetskyi.testnet
```

## CLI Usage

Usage examples assume that you've `dev-deploy`ed popskl contract.

### Prepare

Set accounts:
```bash
ACCOUNT=vchernetskyi.testnet
CONTRACT=popskl.$ACCOUNT
```

Build helper script(s):
```bash
cd scripts
npm install
npx tsc
```

Generate decoded proof:
```bash
LOCATION="My Custom Location"
SECRET=$(node popskl-helper.js secret)
HASH=$(node popskl-helper.js hash "$LOCATION" "$SECRET")
```

### Interact

Issue new location proof:
```bash
# without timeout
near call $CONTRACT store_proof \
    --args "{\"hash\": \"$HASH\"}" \
    --accountId $ACCOUNT \
    --depositYocto 1060000000000000000000

# without timeout in seconds
near call $CONTRACT store_proof \
    --args "{\"hash\": \"$HASH\", \"timeout\": 60}" \
    --accountId $ACCOUNT \
    --depositYocto 1140000000000000000000
```
**Note:** `store_proof` charges for proof storage; any excessive attached deposit is refunded.

Terminate private location proof:
```bash
near call $CONTRACT terminate_proof \
    --args "{\"hash\": \"$HASH\"}" \
    --accountId $ACCOUNT
```
