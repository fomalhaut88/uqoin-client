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


/// Try validators sequentionally.
#[macro_export]
macro_rules! try_validators {
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
