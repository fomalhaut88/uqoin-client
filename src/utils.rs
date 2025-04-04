use std::io::Write;

use rpassword::read_password;
use uqoin_core::state::OrderCoinsMap;


/// Represent string as bytes array with fixed size.
pub fn str_to_bytes<const N: usize>(s: &str) -> [u8; N] {
    let bytes = s.as_bytes();
    let size = std::cmp::min(bytes.len(), N);
    let mut buffer = [0u8; N];
    buffer[..size].clone_from_slice(&bytes[..size]);
    buffer
}


// /// Represent bytes as string.
// pub fn bytes_to_str(bytes: &[u8]) -> &str {
//     std::str::from_utf8(bytes).unwrap().trim_end_matches('\0')
// }


/// Ask user for password.
pub fn require_password() -> std::io::Result<String> {
    print!("Password: ");
    std::io::stdout().flush()?;
    read_password()
}


/// Get total balance of coins_map.
pub fn get_total_balance(coins_map: &OrderCoinsMap) -> u128 {
    let mut balance = 0;
    for (order, coins) in coins_map.iter() {
        balance += (coins.len() as u128) << *order as usize;
    }
    balance
}


/// Normalize path and ensure the parent directory exists.
pub fn ensure_location(path: &str) -> std::io::Result<String> {
    // Normalize path
    let path_buff = if path.starts_with("~/") {
        let mut pb = home::home_dir()
            .expect("Unable to access the user directory.");
        pb.push(&path[2..]);
        pb
    } else {
        path.into()
    };

    // Ensure the directory
    let parent_dir = path_buff.parent().unwrap();
    if !std::fs::exists(&parent_dir)? {
        std::fs::create_dir_all(parent_dir)?;
    }

    // Return
    Ok(path_buff.display().to_string())
}


/// Try first successful validator.
#[macro_export]
macro_rules! try_first_validator {
    ($validators:expr, $func:ident $(, $arg:expr)*) => {
        {
            let mut res = None;
            for validator in $validators.iter() {
                if let Ok(r) = $func($($arg,)* validator) {
                    res = Some(r);
                    break;
                }
            }
            res
        }
    }
}


/// Try all validators. Macros returns count of successful calls.
#[macro_export]
macro_rules! try_all_validators {
    ($validators:expr, $func:ident $(, $arg:expr)*) => {
        {
            let mut count: usize = 0;
            for validator in $validators.iter() {
                if $func($($arg,)* validator).is_ok() {
                    count += 1;
                }
            }
            count
        }
    }
}
