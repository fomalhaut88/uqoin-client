use std::sync::{Arc, RwLock};

use serde::Deserialize;
use serde_json;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::block::Block;
use uqoin_core::transaction::Transaction;
use uqoin_core::coin::{coin_order, coin_random};

use crate::utils::*;
use crate::appdata::load_with_password;
use crate::api::request_send;


pub fn mining(wallet: &str, address: &str, coin: &str, 
              fee: Option<&str>, threads: usize) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Get current state
    let block_hash = request_last_block(&appdata.get_validators()[0])?.hash;
    let block_hash_arc = Arc::new(RwLock::new(block_hash));

    // let coins_map = request_coins_map(wallet, &appdata.get_validators()[0])?;
    // let coins_map_arc = Arc::new(RwLock::new(coins_map));

    const MINING_CHUNK: usize = 1000000;

    // Loop for threads
    for _ in 0..threads {
        // Options to pass
        let schema = Schema::new();
        let miner = U256::from_hex(wallet);
        let address = U256::from_hex(address);
        let min_order = get_order_by_symbol(coin);
        let min_order_fee = fee.map(|s| get_order_by_symbol(s));
        let block_hash_arc = Arc::clone(&block_hash_arc);
        let wallet_key = appdata.get_wallet_key(wallet).unwrap().clone();
        let validator_root = appdata.get_validators()[0].clone();

        // Minimum mining order
        let min_order_mining = std::cmp::min(min_order,
                                             min_order_fee.unwrap_or(256));

        // Spawn a thread
        std::thread::spawn(move || {
            // Random generator
            let mut rng = rand::rng();

            loop {
                // Clone block_hash_prev
                let block_hash_prev = block_hash_arc.read().unwrap().clone();

                // Try to mine some coins
                let mut coins = (0..MINING_CHUNK)
                    .map(|_| coin_random(&mut rng, &block_hash_prev, &miner))
                    .filter(|coin| coin_order(coin, &block_hash_prev, &miner) 
                                   >= min_order_mining)
                    .collect::<Vec<U256>>();

                // Try to create mining groups
                let groups: Vec<Vec<Transaction>> = 
                    if min_order_fee.is_some() {
                        // TODO: Implement a better algorithm that takes coins
                        // from the wallet so more expensive coins have a chance
                        // to be mined using existing fee (not mined at the
                        // same time).
                        // Likely, there will be only one expensive coin so it
                        // is a good idea to mine it using the easiest fee -
                        // from the wallet. This improvement covers most
                        // frequent cases, the others can be reached by more
                        // cores, GPU and other hardware optimizations.

                        // Sort coins by order decreasingly
                        coins.sort_by_key(|coin| 
                            255 - coin_order(coin, &block_hash_prev, &miner)
                        );

                        // Half size
                        let half_size = coins.len() / 2;

                        // Collect coins into transaction pairs
                        (0..half_size).map(|i|
                            vec![
                                Transaction::build(&mut rng, coins[i].clone(), 
                                                   address.clone(), &wallet_key, 
                                                   &schema),
                                Transaction::build(&mut rng, 
                                                   coins[half_size + i].clone(), 
                                                   address.clone(), &wallet_key, 
                                                   &schema),
                            ]
                        ).collect()
                    } else {
                        coins.into_iter().map(|coin| 
                            vec![
                                Transaction::build(&mut rng, coin, 
                                                   address.clone(), &wallet_key, 
                                                   &schema)
                            ]
                        ).collect()
                    };

                // Send groups to the node
                for group in groups.into_iter() {
                    let validator_root = validator_root.clone();
                    std::thread::spawn(move || {
                        request_send(&group, &validator_root).unwrap();
                    });
                }
            }
        });
    }

    // Update state thread
    loop {
        const BLOCK_HASH_UPDATE_TIMEOUT_MILLIS: u64 = 5000;

        std::thread::sleep(
            std::time::Duration::from_millis(BLOCK_HASH_UPDATE_TIMEOUT_MILLIS));

        *block_hash_arc.write().unwrap() = 
            request_last_block(&appdata.get_validators()[0])?.hash;

        // *coins_map_arc.write().unwrap() = 
        //     request_coins_map(wallet, &appdata.get_validators()[0])?;
    }
}


#[allow(dead_code)]
#[derive(Deserialize)]
pub struct BlockData {
    pub bix: u64,
    pub block: Block,
    pub transactions: Option<Vec<Transaction>>,
}


pub fn request_last_block(validator_root: &str) -> std::io::Result<Block> {
    let url = format!("{}/blockchain/block", validator_root);
    let text = reqwest::blocking::get(url).unwrap().text().unwrap();
    let data: BlockData = serde_json::from_str(&text)?;
    Ok(data.block)
}
