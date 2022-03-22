#!/usr/bin/env bash
set -e

echo
echo 'About to call confirm_code() on the contract'
echo near call \$CONTRACT play --account_id \$GUEST \$1
echo
echo \$CONTRACT is $CONTRACT
echo \$GUEST is $GUEST
echo \$1 is [ $1 ]
echo
near call $CONTRACT confirm_code '{"code":"'$1'"}' --account_id $GUEST
