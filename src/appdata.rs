use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result as IOResult};

use rand::Rng;
use serde::{Serialize, Deserialize};
use aes::Aes128;
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::seed::{Seed, Mnemonic};

use crate::utils::*;


pub const APPDATA_PATH: &str = "./tmp/appdata.aes";

pub const VALIDATORS_DEFAULT: [&str; 4] = [
    "http://localhost:8080",
    "http://localhost:8081",
    "http://localhost:8082",
    "http://localhost:8083",
];


/// Load AppData instance requiring user password.
pub fn load_with_password() -> std::io::Result<(AppData, String)> {
    let password = require_password()?;
    let appdata = AppData::load(APPDATA_PATH, &password)?;
    Ok((appdata, password))
}


/// Application data that keeps the seed, wallet keys and validators.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    seed: U256,
    wallets_map: HashMap<String, U256>,
    wallets_seq: Vec<String>,
    validators: Vec<String>,
}


impl AppData {
    /// New AppData instance from raw fields.
    pub fn new(seed: U256, wallets_map: HashMap<String, U256>, 
               wallets_seq: Vec<String>, validators: Vec<String>) -> Self {
        Self { seed, wallets_map, wallets_seq, validators }
    }

    // /// Set default validators.
    // pub fn set_default_validators(&mut self) {
    //     self.validators = 
    //         VALIDATORS_DEFAULT.iter().map(|v| v.to_string()).collect();
    // }

    /// Create an empty AppData instance.
    pub fn create_empty() -> Self {
        Self::new(U256::from(0), HashMap::new(), Vec::new(),
                  VALIDATORS_DEFAULT.iter().map(|v| v.to_string()).collect())
    }

    /// Generate an AppData instance with a random seed.
    pub fn create_random<R: Rng>(rng: &mut R) -> Self {
        let seed = Seed::random(rng);
        Self::new(seed.value(), HashMap::new(), Vec::new(),
                  VALIDATORS_DEFAULT.iter().map(|v| v.to_string()).collect())
    }

    /// Create an AppData instance from mnemonic phrase (12 words).
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Self {
        let seed = Seed::from_mnemonic(mnemonic);
        Self::new(seed.value(), HashMap::new(), Vec::new(),
                  VALIDATORS_DEFAULT.iter().map(|v| v.to_string()).collect())
    }

    /// Load encrypted AppData instance from the file using password.
    pub fn load(path: &str, password: &str) -> IOResult<Self> {
        // Initialize cipher with the password
        let key = GenericArray::from(str_to_bytes::<16>(password));
        let cipher = Aes128::new(&key);

        // Read encrypted data from the file
        let encrypted = std::fs::read(path)?;

        // Prepare blocks to decrypt
        let blocks: Vec<_> = encrypted.chunks(16)
            .map(|chunk| GenericArray::from(
                TryInto::<[u8; 16]>::try_into(chunk).unwrap()
            )).collect();

        // Decrypt the blocks
        let mut decrypted = blocks.clone();
        cipher.decrypt_blocks(&mut decrypted);

        // Concatenate data
        let data = String::from_utf8(decrypted.concat())
            .unwrap_or("".to_string());

        // If data was not decoded with UTF-8, likely the password is wrong
        if data.is_empty() {
            Err(Error::new(ErrorKind::PermissionDenied, "password"))
        } else {
            // Deserialize the structure
            let instance = serde_json::from_str(&data.trim_end_matches('\0'))?;

            // Return the instance
            Ok(instance)
        }
    }

    /// Save AppData instance into a file encrypted with password.
    pub fn save(&self, path: &str, password: &str) -> IOResult<()> {
        // Initialize cipher with the password
        let key = GenericArray::from(str_to_bytes::<16>(password));
        let cipher = Aes128::new(&key);

        // Prepare data as bytes
        let mut data: Vec<u8> = serde_json::to_string(self)?
            .bytes().collect();
        let size = data.len().next_multiple_of(16);
        data.resize(size, 0);

        // Prepare blocks to encrypt
        let blocks: Vec<_> = data.chunks(16)
            .map(|chunk| GenericArray::from(
                TryInto::<[u8; 16]>::try_into(chunk).unwrap()
            )).collect();

        // Encrypt the blocks
        let mut encrypted = blocks.clone();
        cipher.encrypt_blocks(&mut encrypted);

        // Save data to the file
        std::fs::write(path, encrypted.concat())?;

        Ok(())
    }

    /// Return `true` if AppData is empty else `false`.
    pub fn is_empty(&self) -> bool {
        self.seed == U256::from(0)
    }

    /// Check if AppData is not empty.
    pub fn check_not_empty(&self) -> std::io::Result<()> {
        if self.is_empty() {
            Err(Error::new(ErrorKind::Other, "empty appdata"))
        } else {
            Ok(())
        }
    }

    /// Get mnemonic phrase (12 words).
    pub fn mnemonic(&self) -> Mnemonic {
        let seed = Seed::from_value(&self.seed);
        seed.mnemonic()
    }

    /// Iterate wallets (their public keys).
    pub fn get_wallets(&self) -> &[String] {
        &self.wallets_seq
    }

    /// Get a private key of the wallet.
    pub fn get_wallet_key(&self, wallet: &str) -> Option<&U256> {
        self.wallets_map.get(wallet)
    }

    /// Generate `count` more wallets.
    pub fn more_wallets(&mut self, count: usize, schema: &Schema) {
        let seed = Seed::from_value(&self.seed);
        for key in seed.gen_keys(schema).skip(self.wallets_seq.len())
                       .take(count) {
            let public = schema.get_public(&key).to_hex();
            self.wallets_seq.push(public.clone());
            self.wallets_map.insert(public, key);
        }
    }

    /// Get validators.
    pub fn get_validators(&self) -> &[String] {
        &self.validators
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load() {
        let appdata = AppData::new(
            U256::from(25), 
            HashMap::from([("key1".to_string(), U256::from(15))]), 
            vec!["key1".to_string()],
            vec![
                "http://localhost:8080".to_string(),
                "http://localhost:8081".to_string(),
                "http://localhost:8082".to_string(),
                "http://localhost:8083".to_string(),
            ]
        );
        appdata.save("./tmp/test.aes", "qwerty123").unwrap();

        let appdata2 = AppData::load("./tmp/test.aes", "qwerty123").unwrap();
        assert_eq!(appdata2.seed, appdata.seed);
        assert_eq!(appdata2.wallets_map, appdata.wallets_map);
        assert_eq!(appdata2.validators, appdata.validators);
    }
}
