// Clap tutorial: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html

use clap::{Args, Subcommand, Parser};

mod utils;
mod password;
mod account;


/// Uqoin-client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}


#[derive(Subcommand, Debug)]
pub enum Command {
    /// Password management in the application.
    Password {
        #[command(flatten)]
        action: PasswordAction,
    },

    /// Basic account management.
    Account {
        #[command(flatten)]
        action: AccountAction,
    },

    /// Show seed.
    Seed,

    /// Show private key.
    Private {
        /// Wallet address.
        #[arg(short, long)]
        wallet: String,
    },

    /// Show wallets and balance.
    Wallets {
        /// Generate 10 new wallets.
        #[arg(short, long)]
        more: bool,
    },

    /// Send transfer transaction.
    Transfer {
        /// Wallet that keeps the coin.
        #[arg(short, long)]
        wallet: String,

        /// Receiver address.
        #[arg(short, long)]
        addr: String,

        /// Coin symbol.
        #[arg(short, long)]
        coin: String,

        /// Fee coin symbol.
        #[arg(short, long)]
        fee: Option<String>,
    },

    /// Send split transaction.
    Split {
        /// Wallet that keeps the coin.
        #[arg(short, long)]
        wallet: String,

        /// Coin symbol.
        #[arg(short, long)]
        coin: String,

        /// Fee coin symbol.
        #[arg(short, long)]
        fee: Option<String>,
    },

    /// Send merge transaction.
    Merge {
        /// Wallet that keeps the coins.
        #[arg(short, long)]
        wallet: String,

        /// Symbol of requested coin.
        #[arg(short, long)]
        coin: String,

        /// Fee coin symbol.
        #[arg(short, long)]
        fee: Option<String>,
    },

    /// Run mining.
    Mine {
        /// Wallet that signs the coins.
        #[arg(short, long)]
        wallet: String,

        /// Receiver address.
        #[arg(short, long)]
        addr: Option<String>,

        /// Munumum symbol of mined coins.
        #[arg(short, long)]
        coin: String,

        /// Fee coin symbol (it is also being mined if not exists).
        #[arg(short, long)]
        fee: Option<String>,

        /// Number of threads.
        #[arg(short, long, default_value_t = 1)]
        threads: u16,
    },
}


#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct PasswordAction {
    /// Create a new password.
    #[arg(short, long)]
    new: bool,

    /// Change existing password.
    #[arg(short, long)]
    change: bool,

    /// Drop the password and all data (use it carefully).
    #[arg(short, long)]
    drop: bool,
}


#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct AccountAction {
    /// Create a new account.
    #[arg(short, long)]
    new: bool,

    /// Initialize a new account.
    #[arg(short, long)]
    init: bool,

    /// Drop the account.
    #[arg(short, long)]
    drop: bool,
}


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Password { action } => {
            if action.new {
                password::new();
            }
            if action.change {
                password::change();
            }
            if action.drop {
                password::drop();
            }
        },
        Command::Account { action } => {
            if action.new {
                account::new();
            }
            if action.init {
                account::init();
            }
            if action.drop {
                account::drop();
            }
        },
        _ => {},
    }
}
