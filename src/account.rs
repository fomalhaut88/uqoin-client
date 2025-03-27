use std::io::BufRead;

use uqoin_core::schema::Schema;
use uqoin_core::seed::Mnemonic;

use crate::utils::require_password;
use crate::appdata::{AppData, APPDATA_PATH, load_with_password, 
                     ensure_appdata_path};


pub fn password_new() -> std::io::Result<()> {
    let appdata_path = ensure_appdata_path(APPDATA_PATH)?;
    if std::fs::exists(appdata_path)? {
        println!("Account already exists. Drop it first to create a new one.");
    } else {
        println!("Please, enter desirable password.");
        let password = require_password()?;
        println!("");

        println!("Good. Now enter the same password again to confirm.");
        let password_confirm = require_password()?;
        println!("");

        if password != password_confirm {
            println!("Passwords do not match. Try again.");
        } else {
            let appdata = AppData::create_empty();
            appdata.save(&password)?;
            println!("Password has been set successfully.");
        }
    }
    Ok(())
}


pub fn password_change() -> std::io::Result<()> {
    let appdata_path = ensure_appdata_path(APPDATA_PATH)?;
    if std::fs::exists(appdata_path)? {
        println!("Please, enter current password.");
        let password = require_password()?;
        println!("");

        println!("Please, enter new password.");
        let password_new = require_password()?;
        println!("");

        println!("Good. Now enter new password again to confirm.");
        let password_confirm = require_password()?;
        println!("");

        if password_new != password_confirm {
            println!("Passwords do not match. Try again.");
        } else {
            let appdata = AppData::load(&password)?;
            appdata.save(&password_new)?;
            println!("Password has been changed successfully.");
        }
    } else {
        println!("Account does not exist yet.");
    }
    Ok(())
}


pub fn new_random() -> std::io::Result<()> {
    let password = require_password()?;
    let appdata = AppData::load(&password)?;

    if appdata.is_empty() {
        let mut rng = rand::rng();
        let schema = Schema::new();
        let mut appdata = AppData::create_random(&mut rng);
        appdata.more_wallets(10, &schema);
        appdata.save(&password)?;
        println!("A new account has been initialized with a random seed.");
    } else {
        println!("Account is already initialized.");
    }
    Ok(())
}


pub fn new_existing() -> std::io::Result<()> {
    let password = require_password()?;
    let appdata = AppData::load(&password)?;

    if appdata.is_empty() {
        println!("Enter mnemonic phrase (12 words):");

        let mut phrase = String::new();
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut phrase)?;
        let words = phrase.trim().to_lowercase().split_whitespace()
            .map(|word| word.to_string()).collect::<Vec<String>>();

        if words.len() != 12 {
            println!("Invalid mnemonic phrase.");
        } else {
            let mnemonic: Mnemonic = words.try_into().unwrap();
            let schema = Schema::new();
            let mut appdata = AppData::from_mnemonic(&mnemonic);
            appdata.more_wallets(10, &schema);
            appdata.save(&password)?;
            println!("Account has been initialized with mnemonic phrase.");
        }
    } else {
        println!("Account is already initialized.");
    }
    Ok(())
}


pub fn seed() -> std::io::Result<()> {
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;
    let mnemonic = appdata.mnemonic();
    println!("{}", mnemonic.join(" "));
    Ok(())
}


pub fn drop() -> std::io::Result<()> {
    println!("Are you sure you want to delete all the application data?");
    load_with_password()?;
    let appdata_path = ensure_appdata_path(APPDATA_PATH)?;
    if std::fs::exists(&appdata_path)? {
        std::fs::remove_file(&appdata_path)?;
        println!("Account has been fully removed.");
    } else {
        println!("Account does not exist yet.");
    }
    Ok(())
}
