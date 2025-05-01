use clap::{Args, Subcommand, Parser};

mod utils;
mod appdata;
mod account;
mod wallet;
mod api;
mod mining;
mod node;


/// Uqoin-client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}


#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct AccountPasswordAction {
    /// Create a new password.
    #[arg(short, long)]
    new: bool,

    /// Change existing password.
    #[arg(short, long)]
    change: bool,
}


#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct AccountNewAction {
    /// Create a random account.
    #[arg(short, long)]
    random: bool,

    /// Create an account from mnemonic phrase (12 words).
    #[arg(short, long)]
    existing: bool,
}


#[derive(Subcommand, Debug)]
pub enum AccountCommand {
    /// Password management.
    Password {
        #[command(flatten)]
        action: AccountPasswordAction,
    },

    /// Create a new account (random or from mnemonic).
    New {
        #[command(flatten)]
        action: AccountNewAction,
    },

    /// Show mnemonic phrase.
    Seed,

    /// Drop all data.
    Drop,
}


#[derive(Subcommand, Debug)]
pub enum WalletCommand {
    /// List available wallets.
    List,

    /// Create more wallets.
    More {
        #[arg(short, long, default_value_t = 10)]
        count: usize,
    },

    /// Show wallet private key.
    Private {
        /// Wallet address (public key).
        #[arg(short, long)]
        wallet: String,
    },
}


#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    /// Show available balance and coins.
    Balance {
        /// Wallet address.
        #[arg(short, long)]
        wallet: String,

        /// Show coins map.
        #[arg(short, long)]
        coins: bool,

        /// Show available coin numbers.
        #[arg(short, long)]
        detailed: bool,

        /// Units of the balance (A, B, C, etc).
        #[arg(short, long)]
        unit: Option<char>,
    },

    /// Send coin to address.
    Send {
        /// Sender address (wallet from).
        #[arg(short, long)]
        wallet: String,

        /// Receiver address (wallet to).
        #[arg(short, long)]
        address: String,

        /// Coin (symbol or number).
        #[arg(short, long)]
        coin: String,

        /// Fee coin (symbol or number).
        #[arg(short, long)]
        fee: Option<String>,
    },

    /// Split coin.
    Split {
        /// Wallet address.
        #[arg(short, long)]
        wallet: String,

        /// Coin to split (symbol or number).
        #[arg(short, long)]
        coin: String,

        /// Fee coin (symbol or number).
        #[arg(short, long)]
        fee: Option<String>,
    },

    /// Merge coin.
    Merge {
        /// Wallet address.
        #[arg(short, long)]
        wallet: String,

        /// Desirable coin to merge (symbol).
        #[arg(short, long)]
        coin: String,

        /// Fee coin (symbol or number).
        #[arg(short, long)]
        fee: Option<String>,
    },
}


#[derive(Subcommand, Debug)]
pub enum NodeCommand {
    /// List URLs of known nodes.
    List,

    /// Add a URL of new node.
    Add {
        /// URL of the node.
        #[arg(short, long)]
        node: String,
    },

    /// Remove the node by url.
    Remove {
        /// URL of the node.
        #[arg(short, long)]
        node: String,
    },

    /// Move the node to the position in the list.
    Move {
        /// URL of the node.
        #[arg(short, long)]
        node: String,

        /// Desirable position in the list (starting with 1).
        #[arg(short, long)]
        pos: usize,
    },

    /// Set node list to its default state.
    Default,

    /// Fetch known nodes from the given node (or all ones if the node is not 
    /// specified).
    Fetch {
        /// URL of the node.
        #[arg(short, long)]
        node: Option<String>,
    },
}


#[derive(Subcommand, Debug)]
pub enum Command {
    /// Basic account management.
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },

    /// Wallet operations.
    Wallet {
        #[command(subcommand)]
        command: WalletCommand,
    },

    /// Operations in the net.
    Api {
        #[command(subcommand)]
        command: ApiCommand,
    },

    /// Run mining.
    Mining {
        /// Wallet to sign new coins.
        #[arg(short, long)]
        wallet: String,

        /// Receiver wallet address.
        #[arg(short, long)]
        address: Option<String>,

        /// Fee coin (symbol).
        #[arg(short, long)]
        fee: String,

        /// Number of threads.
        #[arg(short, long, default_value_t = 1)]
        threads: usize,
    },

    /// Node management.
    Node {
        #[command(subcommand)]
        command: NodeCommand,
    },
}


fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Account { command } => {
            match command {
                AccountCommand::Password { action } => {
                    if action.new {
                        account::password_new()?;
                    }
                    if action.change {
                        account::password_change()?;
                    }
                },
                AccountCommand::New { action } => {
                    if action.random {
                        account::new_random()?;
                    }
                    if action.existing {
                        account::new_existing()?;
                    }
                },
                AccountCommand::Seed => {
                    account::seed()?;
                },
                AccountCommand::Drop => {
                    account::drop()?;
                },
            }
        },

        Command::Wallet { command } => {
            match command {
                WalletCommand::List => {
                    wallet::list()?;
                },
                WalletCommand::More { count } => {
                    wallet::more(count)?;
                },
                WalletCommand::Private { wallet } => {
                    wallet::private(&wallet)?;
                },
            }
        },

        Command::Api { command } => {
            match command {
                ApiCommand::Balance { wallet, coins, detailed, unit } => {
                    api::balance(&wallet, coins, detailed, unit)?;
                },
                ApiCommand::Send { wallet, address, coin, fee } => {
                    api::send(&wallet, &address, &coin, fee.as_deref())?;
                },
                ApiCommand::Split { wallet, coin, fee } => {
                    api::split(&wallet, &coin, fee.as_deref())?;
                },
                ApiCommand::Merge { wallet, coin, fee } => {
                    api::merge(&wallet, &coin, fee.as_deref())?;
                },
            }
        },

        Command::Mining { wallet, address, fee, threads } => {
            mining::mining(&wallet, address.as_deref(), &fee, threads)?;
        },

        Command::Node { command } => {
            match command {
                NodeCommand::List => {
                    node::list()?;
                },
                NodeCommand::Add { node } => {
                    node::add(&node)?;
                },
                NodeCommand::Remove { node } => {
                    node::remove(&node)?;
                },
                NodeCommand::Move { node, pos } => {
                    node::r#move(&node, pos)?;
                },
                NodeCommand::Default => {
                    node::default()?;
                },
                NodeCommand::Fetch { node } => {
                    node::fetch(node.as_deref())?;
                },
            }
        },
    }

    Ok(())
}
