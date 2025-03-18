use std::collections::{HashSet, HashMap};
use std::io::{Error, ErrorKind, Result as IOResult};
use std::io::BufRead;

use rand::Rng;
use serde::{Serialize, Deserialize};
use serde_json;
use aes::Aes128;
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::state::OrderCoinsMap;
use uqoin_core::seed::{Mnemonic, Seed};

use crate::utils::*;


pub const ACCOUNT_PATH: &str = "./tmp/account.aes";


pub fn new() -> std::io::Result<()> {
    let password = get_password()?;
    let account = Account::load(ACCOUNT_PATH, &password)?;

    if account.is_empty() {
        let mut rng = rand::rng();
        let schema = Schema::new();
        let mut account = Account::create_random(&mut rng);
        account.more_wallets(10, &schema);
        account.save(ACCOUNT_PATH, &password)?;
        println!("A new account has been initialized with a random seed.");
    } else {
        println!("Account is already initialized.");
    }
    Ok(())
}


pub fn init() -> std::io::Result<()> {
    // Example mnemonic:
    // accident child alpha chief mountain useless long basket zebra pole equip strike

    let password = get_password()?;
    let account = Account::load(ACCOUNT_PATH, &password)?;

    if account.is_empty() {
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
            let mut account = Account::from_mnemonic(&mnemonic);
            account.more_wallets(10, &schema);
            // TODO: Load coins from blockchain
            account.save(ACCOUNT_PATH, &password)?;
            println!("Account has been initialized with mnemonic phrase.");
        }
    } else {
        println!("Account is already initialized.");
    }
    Ok(())
}


pub fn clean() -> std::io::Result<()> {
    let password = get_password()?;
    Account::load(ACCOUNT_PATH, &password)?;
    Account::create_empty().save(ACCOUNT_PATH, &password)?;
    println!("All account data has been removed.");
    Ok(())
}


pub fn seed() -> std::io::Result<()> {
    let password = get_password()?;
    let account = Account::load(ACCOUNT_PATH, &password)?;
    if account.is_empty() {
        println!("Account is not initialized.");
    } else {
        let mnemonic = account.mnemonic();
        println!("{}", mnemonic.join(" "));
    }
    Ok(())
}


pub fn private(wallet: &str) -> std::io::Result<()> {
    let password = get_password()?;
    let account = Account::load(ACCOUNT_PATH, &password)?;
    if account.is_empty() {
        println!("Account is not initialized.");
    } else {
        if let Some(key) = account.get_wallet_key(wallet) {
            println!("{}", key.to_hex());
        } else {
            println!("Wallet not found.");
        }
    }
    Ok(())
}


pub fn wallets() -> std::io::Result<()> {
    let password = get_password()?;
    let account = Account::load(ACCOUNT_PATH, &password)?;
    if account.is_empty() {
        println!("Account is not initialized.");
    } else {
        for wallet in account.get_wallets() {
            let balance = account.get_balance(wallet).unwrap();
            println!("{} : {}", wallet, balance.to_decimal());
        }
    }
    Ok(())
}


pub fn coins(wallet: &str) -> std::io::Result<()> {
    let password = get_password()?;
    let account = Account::load(ACCOUNT_PATH, &password)?;
    if account.is_empty() {
        println!("Account is not initialized.");
    } else {
        if let Some(coins_map) = account.get_coins(wallet) {
            if coins_map.is_empty() {
                println!("Empty.");
            } else {
                for (order, coins) in coins_map.iter() {
                    println!("{} - {}", order, coins.len());
                }
            }
        } else {
            println!("Wallet not found.");
        }
    }
    Ok(())
}


/// Account state structure that keeps the seed, wallet keys, coins and 
/// validators.
#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    seed: U256,
    wallets: HashMap<String, U256>,
    coins: HashMap<String, OrderCoinsMap>,
    validators: Vec<String>,
}


impl Account {
    /// New account from raw fields.
    pub fn new(seed: U256, wallets: HashMap<String, U256>, 
               coins: HashMap<String, OrderCoinsMap>, 
               validators: Vec<String>) -> Self {
        Self { seed, wallets, coins, validators }
    }

    /// Create empty accound.
    pub fn create_empty() -> Self {
        Self::new(U256::from(0), HashMap::new(), HashMap::new(), Vec::new())
    }

    /// Generate account with a random seed.
    pub fn create_random<R: Rng>(rng: &mut R) -> Self {
        let seed = Seed::random(rng);
        Self::new(seed.value(), HashMap::new(), HashMap::new(), Vec::new())
    }

    /// Create account from mnemonic phrase (12 words).
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Self {
        let seed = Seed::from_mnemonic(mnemonic);
        Self::new(seed.value(), HashMap::new(), HashMap::new(), Vec::new())
    }

    /// Load encrypted account from the file using password.
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

    /// Save account data into the file encrypted with password.
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

    /// Check is account is empty.
    pub fn is_empty(&self) -> bool {
        self.seed == U256::from(0)
    }

    /// Get mnemonic phrase (12 words).
    pub fn mnemonic(&self) -> Mnemonic {
        let seed = Seed::from_value(&self.seed);
        seed.mnemonic()
    }

    /// Iterate wallets (their public keys).
    pub fn get_wallets(&self) -> impl Iterator<Item = &String> {
        self.wallets.keys()
    }

    /// Get a private key of the wallet.
    pub fn get_wallet_key(&self, wallet: &str) -> Option<&U256> {
        self.wallets.get(wallet)
    }

    /// Generate `count` more wallets.
    pub fn more_wallets(&mut self, count: usize, schema: &Schema) {
        let seed = Seed::from_value(&self.seed);
        for key in seed.gen_keys(schema).skip(self.wallets.len()).take(count) {
            let public = schema.get_public(&key).to_hex();
            self.coins.insert(public.clone(), OrderCoinsMap::new());
            self.wallets.insert(public, key);
        }
    }

    /// Get coins map of the wallet.
    pub fn get_coins(&self, wallet: &str) -> Option<&OrderCoinsMap> {
        self.coins.get(wallet)
    }

    /// Get total balance of the wallet.
    pub fn get_balance(&self, wallet: &str) -> Option<U256> {
        let mut balance = U256::from(0);
        for (order, coins) in self.coins.get(wallet)? {
            balance += &(&U256::from(coins.len()) << *order as usize)
        }
        Some(balance)
    }

    /// Pop coin of given order from the wallet.
    pub fn pop_coin(&mut self, wallet: &str, order: u64) -> Option<U256> {
        let coins = self.coins.get_mut(wallet)?.get_mut(&order)?;
        let coin = coins.iter().next()?.clone();
        coins.remove(&coin);
        Some(coin)
    }

    /// Push coin to the given wallet.
    pub fn push_coin(&mut self, wallet: &str, order: u64, 
                     coin: U256) -> Option<()> {
        let map = self.coins.get_mut(wallet)?;
        if !map.contains_key(&order) {
            map.insert(order, HashSet::new());
        }
        map.get_mut(&order).unwrap().insert(coin);
        Some(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load() {
        let account = Account::new(
            U256::from(25), 
            HashMap::from([("key1".to_string(), U256::from(15))]), 
            HashMap::new(), vec!["yesman".to_string()]
        );
        account.save("./tmp/test.aes", "qwerty123").unwrap();

        let account2 = Account::load("./tmp/test.aes", "qwerty123").unwrap();
        assert_eq!(account2.seed, account.seed);
        assert_eq!(account2.wallets, account.wallets);
        assert_eq!(account2.coins, account.coins);
        assert_eq!(account2.validators, account.validators);
    }
}
