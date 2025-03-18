use rpassword::read_password;
use std::io::Write;


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
pub fn get_password() -> std::io::Result<String> {
    print!("Password: ");
    std::io::stdout().flush()?;
    read_password()
}
