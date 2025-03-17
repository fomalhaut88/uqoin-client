use std::collections::HashMap;

use uqoin_core::utils::U256;
use uqoin_core::state::OrderCoinsMap;

use crate::utils::*;


pub fn new() {
    println!("new");
}


pub fn init() {
    println!("init");
}


pub fn drop() {
    println!("drop");
}


pub struct Account {
    seed: U256,
    wallets: HashMap<U256, U256>,
    coins: HashMap<U256, OrderCoinsMap>,
    validators: Vec<String>,
}


impl Account {
    pub fn new(seed: U256, wallets: HashMap<U256, U256>, 
               coins: HashMap<U256, OrderCoinsMap>) -> Self {
        unimplemented!()
    }

    pub fn create() -> Self {
        unimplemented!()
    }

    pub fn from_file(path: &str, password: &str) -> Option<Self> {
        unimplemented!()
    }

    pub fn from_seed(seed_phrase: SeedPhrase) -> Option<Self> {
        unimplemented!()
    }

    pub fn save(&self, password: &str) {
        unimplemented!()
    }

    pub fn seed_phrase(&self) -> SeedPhrase {
        unimplemented!()
    }

    pub fn get_wallets(&self) -> impl Iterator<Item = &U256> {
        self.wallets.keys()
    }

    pub fn get_wallet_key(&self, wallet: &U256) -> Option<&U256> {
        self.wallets.get(wallet)
    }

    pub fn more_wallets(&mut self, count: usize) {
        unimplemented!()
    }

    pub fn get_coins(&self, wallet: &U256) -> Option<&OrderCoinsMap> {
        self.coins.get(wallet)
    }

    pub fn get_balance(&self, wallet: &U256) -> U256 {
        unimplemented!()
    }

    pub fn pop_coin(&mut self, wallet: &U256, order: u64) -> Option<U256> {
        unimplemented!()
    }

    pub fn push_coin(&mut self, wallet: &U256, coin: U256) {
        unimplemented!()
    }
}
