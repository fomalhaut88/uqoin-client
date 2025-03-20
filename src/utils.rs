use std::io::Write;

use rpassword::read_password;
use uqoin_core::utils::U256;
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
pub fn get_total_balance(coins_map: &OrderCoinsMap) -> U256 {
    let mut balance = U256::from(0);
    for (order, coins) in coins_map.iter() {
        balance += &(&U256::from(coins.len()) << *order as usize);
    }
    balance
}


/// Get coin order by symbol.
pub fn get_order_by_symbol(symbol: &str) -> u64 {
    let letter = symbol.chars().next().unwrap() as u64 - 'A' as u64;
    let number: u64 = symbol[1..].parse().unwrap();
    10 * letter + number.trailing_zeros() as u64
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_order_by_symbol() {
        assert_eq!(get_order_by_symbol("C32"), 25);
        assert_eq!(get_order_by_symbol("D4"), 32);
        assert_eq!(get_order_by_symbol("B1"), 10);
        assert_eq!(get_order_by_symbol("A1"), 0);
        assert_eq!(get_order_by_symbol("Z32"), 255);
    }
}
