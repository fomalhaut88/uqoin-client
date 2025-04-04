# uqoin-client

A command line tool to interact with nodes of Uqoin ecosystem. It supports
working with wallets, sending transactions, mining. The data of the application
is stored on disk encrypted with AES-128 so you need to enter the password
every time to run a command.

## Installation

1. Download the source: `git clone https://github.com/fomalhaut88/uqoin-client.git --depth=1`
2. Compile (you need Rust installed in Nightly toolchain): `cargo build --release`
3. Copy the executable: `sudo cp ./target/release/uqoin-client /usr/bin/uqoin-client`

## Quick start

1. Show help: `uqoin-client help`
2. Create a new password: `uqoin-client account password --new`
3. Create a new account with a random seed phrase (or import an existing one by 
`--existing` instead of `--random`): `uqoin-client account new --random`
4. Print your seed phrase (save it in a safe place!): `uqoin-client account seed`
5. List available wallets: `uqoin-client wallet list`

## Short list of useful commands

| Command | Description |
|---|---|
| `uqoin-client wallet private -w <WALLET>` | Show private key of your wallet `WALLET`. |
| `uqoin-client api balance -w <WALLET> -u D` | Show balance of the wallet `WALLET` (any) in D-units. |
| `uqoin-client api send -w <WALLET1> -a <WALLET2> -c D128 -f D1` | Send coin D128 from `WALLET1` to `WALLET2` with fee coin D1. |
| `uqoin-client api split -w <WALLET> -c D128 -f D1` | Split coin D128 in `WALLET` into D64 and two D32 with fee coin D1. |
| `uqoin-client api merge -w <WALLET> -c D128 -f D1` | Split coins D64 and two D32 in `WALLET` into D128 with fee coin D1. |
| `uqoin-client mining -w <WALLET> -c D1 -f C32 -t 4` | Run mining of the coins greater or eqial D1 and fee greater of equal C32 on 4 cores for the wallet `WALLET`. |

## All commands

| Command | Description |
|---|---|
| `uqoin-client help` | Print help. |
| `uqoin-client -V` | Version of the application. |
| `uqoin-client account password --new` | Create a new password for the stored data. |
| `uqoin-client account password --change` | Change password for the stored data. |
| `uqoin-client account new --random` | Initialize a new account with a random seed. |
| `uqoin-client account new --existing` | Initialize a new account with an existing seed. You will be asked to enter 12 words of your mnemonic phrase. |
| `uqoin-client account seed` | Show 12 words of the seed phrase. Keep it in a safe place and do not show it to anyone. |
| `uqoin-client account drop` | Delete all data. **Use it carefully and do not forget to backup your seed phrase (to show it use `uqoin-client account seed`).** |
| `uqoin-client account seed` | Show 12 words of the seed phrase. Keep it in a safe place and do not show it to anyone. |
| `uqoin-client wallet list` | Show available wallets. |
| `uqoin-client wallet more -c <COUNT>` | Create ` <COUNT>` more wallets (default **10**). |
| `uqoin-client wallet private -w <WALLET>` | Show private key of `<WALLET>`. Do not show it to anybody. |
| `uqoin-client api balance -w <WALLET>` | Show balance of the `<WALLET>`. `-u <UNIT>` - unit of the balance (letter A, B, C, etc), `-c` - coin counts, `-d` - in details. |
| `uqoin-client api send -w <WALLET> -a <ADDR> -c <COIN> -f <FEE>` | Send `<COIN>` (as symbol) from `<WALLET>` (your wallet) to `<ADDR>` (wallet of the receiver) attaching the fee coin symbol `<FEE>` (optional). |
| `uqoin-client api split -w <WALLET> -c <COIN> -f <FEE>` | Split `<COIN>` (as symbol) from `<WALLET>` into 3 coins attaching the fee coin symbol `<FEE>` (optional). |
| `uqoin-client api merge -w <WALLET> -c <COIN> -f <FEE>` | Merge 3 coins from `<WALLET>` into `<COIN>` (as symbol) attaching the fee coin symbol `<FEE>` (optional). |
| `uqoin-client api mining -w <WALLET> -c <COIN> -f <FEE> -t <THREADS>` | Run mining into the `<WALLET>` of coins greater or equal `<COIN>` (as symbol) attaching the fee coin greater or equal to symbol `<FEE>` (optional) in `<THREADS>` (default **1**). |
| `uqoin-client node list` | Show available nodes to request (order matters, most requests are implemented sequentially until first success). |
| `uqoin-client node add -n <NODE>` | Add a `<NODE>` to the end. |
| `uqoin-client node remove -n <NODE>` | Remove `<NODE>`. |
| `uqoin-client node move -n <NODE> -p <POS>` | Move `<NODE>` to the position `<POS>` in the list (starting with 1). |
| `uqoin-client node default` | Restore default nodes. |
| `uqoin-client node fetch -n <NODE>` | Fetch known sync nodes from `<NODE>` if specified or applying to all nodes in the local list. |
