# uqoin-client

A command line tool to interact with nodes of Uqoin ecosystem. It supports
working with wallets, sending transactions, mining. The data of the application
is stored on disk encrypted with AES-128 so you need to enter the password
every time to run a command.

## Quick start

1. Show help: `uqoin-client help`
2. Create a new password: `uqoin-client account password --new`
3. Create a new account with a random seed phrase (or import an existing one by 
`--existing` instead of `--random`): `uqoin-client account new --random`
4. Print your seed phrase (save it in a safe place!): `uqoin-client account seed`
5. List available wallets: `uqoin-client wallet list`

## Useful commands

| Command | Description |
|---|---|
| `uqoin-client wallet private -w <WALLET>` | Show private key of your wallet `WALLET`. |
| `uqoin-client api balance -w <WALLET> -u D` | Show balance of the wallet `WALLET` (any) in D-units. |
| `uqoin-client api send -w <WALLET1> -a <WALLET2> -c D128 -f D1` | Send coin D128 from `WALLET1` to `WALLET2` with fee coin D1. |
| `uqoin-client api split -w <WALLET> -c D128 -f D1` | Split coin D128 in `WALLET` into D64 and two D32 with fee coin D1. |
| `uqoin-client api merge -w <WALLET> -c D128 -f D1` | Split coins D64 and two D32 in `WALLET` into D128 with fee coin D1. |
| `uqoin-client mining -w <WALLET> -c D1 -f C32 -t 4` | Run mining of the coins greater or eqial D1 and fee greater of equal C32 on 4 cores for the wallet `WALLET`. |
