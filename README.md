# uqoin-client

![Top Language](https://img.shields.io/github/languages/top/fomalhaut88/winrk)
![Version](https://img.shields.io/badge/version-v0.1.3-green)
![License](https://img.shields.io/badge/license-MIT-orange)

A command line tool to interact with nodes of Uqoin ecosystem. It supports
working with wallets, sending transactions, mining. The data of the application
is stored on disk encrypted with AES-128 so you need to enter the password
every time to run a command.

## Installation

### For Linux

1. Download the source: `git clone https://github.com/fomalhaut88/uqoin-client.git --depth=1`
2. Compile (you need Rust installed in Nightly toolchain): `cargo build --release`
3. Copy the executable: `sudo cp ./target/release/uqoin-client /usr/bin/uqoin-client`

### For Windows

1. Download the setup file from SourceForge: https://sourceforge.net/projects/uqoin-client/files/latest/download
2. Run the downloaded file and follow the wizard.
3. Restart the computer.

### Build for Windows

1. Download the source: `git clone https://github.com/fomalhaut88/uqoin-client.git --depth=1`
2. Compile (you need Rust installed in Nightly toolchain): `cargo build --release`
3. Pack with [Inno Setup](https://jrsoftware.org/isinfo.php): `iscc uqoin-client-setup.iss`
4. Run the appeared EXE-file `uqoin-client-setup-<VERSION>.exe` and follow the wizard.
5. Restart the computer.

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
| `uqoin-client api balance -w <WALLET> -u E` | Show balance of the wallet `WALLET` (any) in E-units. |
| `uqoin-client api send -w <WALLET1> -a <WALLET2> -c D128 -f D1` | Send coin D128 from `WALLET1` to `WALLET2` with fee coin D1. |
| `uqoin-client api split -w <WALLET> -c D128 -f D1` | Split coin D128 in `WALLET` into D64 and two D32 with fee coin D1. |
| `uqoin-client api merge -w <WALLET> -c D128 -f D1` | Split coins D64 and two D32 in `WALLET` into D128 with fee coin D1. |
| `uqoin-client mining -w <WALLET> -f D1 -t 4` | Run mining of the coins greater D1 and fee equal to D1 on 4 cores for the wallet `WALLET`. |

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
| `uqoin-client api mining -w <WALLET> -a <ADDRESS> -f <FEE> -t <THREADS>` | Run mining from the wallet `<WALLET>` into the `<ADDRESS>` (optional,  it is set to `<WALLET>` if not specified) of coins greater `<FEE>` (as symbol) attaching the fee coin equal to symbol `<FEE>` in `<THREADS>` (default **1**). |
| `uqoin-client node list` | Show available nodes to request (order matters, most requests are implemented sequentially until first success). |
| `uqoin-client node add -n <NODE>` | Add a `<NODE>` to the end. |
| `uqoin-client node remove -n <NODE>` | Remove `<NODE>`. |
| `uqoin-client node move -n <NODE> -p <POS>` | Move `<NODE>` to the position `<POS>` in the list (starting with 1). |
| `uqoin-client node default` | Restore default nodes. |
| `uqoin-client node fetch -n <NODE>` | Fetch known sync nodes from `<NODE>` if specified or applying to all nodes in the local list. |
| `uqoin-client tool gen-key` | Generate a random private key of a wallet. |
| `uqoin-client tool gen-pair` | Generate a random pair of public-private key of a wallet. |
| `uqoin-client tool get-public -k <KEY>` | Get public key (wallet address) from the given private key `<KEY>`. |
| `uqoin-client tool gen-seed` | Generate a random seed phrase (12 words). |
| `uqoin-client tool gen-wallets -s <SEED> -c <COUNT> -o <OFFSET>` | Deterministic generate `<COUNT>` (default **10**) wallets (as public-key pairs) with the offset `<OFFSET>` (default **0**) from given mnemonic `<SEED>`. |
| `uqoin-client tool hash -m <MSG1> -m <MSG2> ...` | Calculate SHA3 hash of 256-bit messages `<MSG1>`, `<MSG2>` ... (in HEX). |
| `uqoin-client tool build-signature -m <MSG> -k <KEY>` | Build EdDSA signature from 256-bit message `<MSG>` (in HEX) and private key `<KEY>`. |
| `uqoin-client tool extract-public -m <MSG> -s <SIGNATURE>` | Extract public key (wallet address) from 256-bit message `<MSG>` (in HEX) and EdDSA signature (512-bit in HEX). |
