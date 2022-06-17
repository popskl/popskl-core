#!/usr/bin/env bash
set -e

echo
echo 'About to call set_cooldown() on the contract'
echo
echo \$CONTRACT is $CONTRACT
echo \$OWNER is $OWNER
echo \$1 is $1
echo
near call $CONTRACT set_cooldown '{"cooldown": '$1'}' --account_id $OWNER
