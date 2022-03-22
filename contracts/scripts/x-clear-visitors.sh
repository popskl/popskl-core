#!/usr/bin/env bash
set -e

echo
echo 'About to call clear_visitor_records() on the contract'
echo
echo \$CONTRACT is $CONTRACT
echo \$OWNER is $OWNER
echo
near call $CONTRACT clear_visitor_records --account_id $OWNER
