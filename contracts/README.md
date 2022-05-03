# PoPskl contract

Contract generates and stores codes, that are then confirmed by the users. 
After confirmation contract stores number of times each user submitted a valid code.

## Deployment

There is a small [deploy script](./scripts/1.deploy.sh). 
It requires:
* `OWNER` env variable (master account for the contract)
* `near` & `yarn` executables

Use it like this:
```shell
# by default deploys to testnet, to deploy to other network:
# export NODE_ENV=local
OWNER=<account name>.testnet ./scripts/1.deploy.sh
```

## Usage

See [scripts](./scripts/) directory for examples of contract usage.

## Terminal-based Demo

- The real application would have the QR code visible on some screen at the point of sale
- the user would open popskl.com to see something like a "pop it!" button that opens a camera view
- once the user device reads the QR code, they are either
  - (a) prompted with confirmation of their pop (because the page just calls the contract automatically once a valid QR code is detected)
  - (b) prompted to login to NEAR wallet
  - (c) setup a NEAR account if they don't already have one

https://drive.google.com/file/d/1_4ZrHy1Ow_iq4N2Av1Y7Jhiud1Cc8Mcz/view

## Known issues

* currently we're just tracking fact that user confirmed our code, but we actually want to prove that user was at some location
* contract is essentially single-threaded (i.e. there is only one valid code at a time)
* we need to add a timeout per each code, to limit time range of location proofs
