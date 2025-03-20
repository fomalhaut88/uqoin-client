use uqoin_core::schema::Schema;

use crate::appdata::{load_with_password, APPDATA_PATH};


pub fn list() -> std::io::Result<()> {
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;
    for wallet in appdata.get_wallets() {
        println!("{}", wallet);
    }
    Ok(())
}


pub fn more(count: usize) -> std::io::Result<()> {
    let (mut appdata, password) = load_with_password()?;
    appdata.check_not_empty()?;
    let schema = Schema::new();
    appdata.more_wallets(count, &schema);
    appdata.save(APPDATA_PATH, &password)?;
    Ok(())
}


pub fn private(wallet: &str) -> std::io::Result<()> {
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;
    if let Some(key) = appdata.get_wallet_key(wallet) {
        println!("{}", key.to_hex());
    } else {
        println!("Wallet not found.");
    }
    Ok(())
}
