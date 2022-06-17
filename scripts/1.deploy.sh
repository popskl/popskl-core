#!/usr/bin/env bash

if [[ -z "$OWNER" ]]; then
    echo "Missing \$OWNER environment variable"
    exit 1
fi

CONTRACT=popskl.$OWNER

echo -e "[1] Recreating $CONTRACT and setting $OWNER as beneficiary\n"

near delete $CONTRACT $OWNER
near create-account $CONTRACT --masterAccount $OWNER

# exit on first error after this point to avoid deploying failed build
set -e

echo --------------------------------------------
echo -e "\n[2] rebuilding the contract (release build)\n"

yarn build:release

echo --------------------------------------------
echo -e "\n[3] redeploying the contract\n"
near deploy $CONTRACT \
    --wasmFile ./build/release/popskl.wasm \
    --initFunction init \
    --initArgs "{\"owner\":\"$OWNER\"}"

echo --------------------------------------------
echo
