use clap::{Subcommand, Parser};
use clap::error::{Error, ErrorKind};

mod utils;
mod password;
mod account;

use crate::utils::*;


/// Uqoin-client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}


#[derive(Subcommand, Debug)]
pub enum Command {
    /// Password management in the application.
    Password {
        /// Create a new password.
        #[arg(short, long)]
        new: bool,

        /// Change existing password.
        #[arg(short, long)]
        change: bool,

        /// Drop the password and all date (use it carefully).
        #[arg(short, long)]
        drop: bool,
    },

    /// Basic account management.
    Account {
        /// Initialize a new account.
        #[arg(short, long)]
        init: bool,

        /// Drop the password and all date (use it carefully).
        #[arg(short, long)]
        drop: bool,
    },
}


impl Command {
    pub fn run(&self) -> ClapResult {
        match self {
            Self::Password { new, change, drop } => {
                Self::check_only_flag(&[new, change, drop])?;
                if *new {
                    password::new();
                }
                if *change {
                    password::change();
                }
                if *drop {
                    password::drop();
                }
            },
            Self::Account { init, drop } => {
                Self::check_only_flag(&[init, drop])?;
                if *init {
                    account::init();
                }
                if *drop {
                    account::drop();
                }
            },
        }
        Ok(())
    }

    fn check_only_flag(flags: &[&bool]) -> ClapResult {
        let count = flags.iter().fold(0, |acc, val| acc + **val as u32);
        if count == 1 {
            Ok(())
        } else {
            Err(Error::raw(ErrorKind::ArgumentConflict, "flags conflict"))
        }
    }
}


fn main() {
    let args = Args::parse();
    let res = args.command.run();
    if let Err(err) = res {
        println!("{:?} {}", err.kind(), err);
    }
}
