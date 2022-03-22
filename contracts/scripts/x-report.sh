#!/usr/bin/env bash
set -e

[ -z "$CONTRACT" ] && echo "Missing \$CONTRACT environment variable" && exit 1
[ -z "$OWNER" ] && echo "Missing \$OWNER environment variable" && exit 1

echo "These are the environment variables being used:"
echo
echo "CONTRACT is [ $CONTRACT ]"
echo "OWNER is [ $OWNER ]"
echo
echo

# is this popskl active? -> bool
echo "near view \$CONTRACT get_active '{}'"
near view $CONTRACT get_active '{}'
echo

echo "near view \$CONTRACT get_owner '{}'"
near view $CONTRACT get_owner '{}'
echo

echo "near view \$CONTRACT get_code '{}'"
near view $CONTRACT get_code '{}'
echo

echo "--------------------------------------------"
echo PoPskl Details
echo "--------------------------------------------"

# who visited last? -> AccountId
echo "near view \$CONTRACT get_last_visitor '{}'"
near call $CONTRACT get_last_visitor '{}' --accountId $OWNER
echo

# list all visitors -> Array<AccountId>
echo "near view \$CONTRACT get_visitors '{}'"
near call $CONTRACT get_visitors '{}' --accountId $OWNER
echo

# get visits for visitor -> u16
echo "near view \$CONTRACT get_visit_count '{}'"
near call $CONTRACT get_visit_count '{"guest":"'$GUEST'"}' --accountId $OWNER
echo

# has GUEST visited before -> bool
echo "near view \$CONTRACT get_has_visited '{\"guest\":\"'\$GUEST'\"}'"
near call $CONTRACT get_has_visited '{"guest":"'$GUEST'"}' --accountId $OWNER
echo
