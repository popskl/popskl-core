#!/usr/bin/env bash
set -e

echo
echo 'About to call toggle_active() on the contract'
echo
echo \$CONTRACT is $CONTRACT
echo \$OWNER is $OWNER
echo
near call $CONTRACT toggle_active --account_id $OWNER
