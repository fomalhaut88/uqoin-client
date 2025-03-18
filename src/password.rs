use crate::utils::*;
use crate::account::{ACCOUNT_PATH, Account};


pub fn new() -> std::io::Result<()> {
    if std::fs::exists(ACCOUNT_PATH)? {
        println!("Account already exists. Drop it first to create a new one.");
    } else {
        println!("Please, enter desirable password.");
        let password = get_password()?;
        println!("");

        println!("Good. Now enter the same password again to confirm.");
        let password_confirm = get_password()?;
        println!("");

        if password != password_confirm {
            println!("Passwords do not match. Try again.");
        } else {
            let account = Account::create_empty();
            account.save(ACCOUNT_PATH, &password)?;
            println!("Password has been set successfully.");
        }
    }
    Ok(())
}


pub fn change() -> std::io::Result<()> {
    if std::fs::exists(ACCOUNT_PATH)? {
        println!("Please, enter current password.");
        let password = get_password()?;
        println!("");

        println!("Please, enter new password.");
        let password_new = get_password()?;
        println!("");

        println!("Good. Now enter new password again to confirm.");
        let password_confirm = get_password()?;
        println!("");

        if password_new != password_confirm {
            println!("Passwords do not match. Try again.");
        } else {
            let account = Account::load(ACCOUNT_PATH, &password)?;
            account.save(ACCOUNT_PATH, &password_new)?;
            println!("Password has been changed successfully.");
        }
    } else {
        println!("Account does not exist yet.");
    }
    Ok(())
}


pub fn drop() -> std::io::Result<()> {
    if std::fs::exists(ACCOUNT_PATH)? {
        std::fs::remove_file(ACCOUNT_PATH)?;
        println!("Account has been fully removed.");
    } else {
        println!("Account does not exist yet.");
    }
    Ok(())
}
