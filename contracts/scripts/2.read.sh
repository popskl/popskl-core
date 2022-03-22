#!/usr/bin/env bash
set -e

echo 'About to call get_code() on the contract'
echo
echo \$CONTRACT is $CONTRACT
echo \$GUEST is $GUEST
echo
result=$(near call $CONTRACT get_code --accountId $GUEST)
a=( ${result[0]} )
CODE=( ${a[23]} )
echo $CODE | yarn qr
echo
echo popskl code is $CODE
