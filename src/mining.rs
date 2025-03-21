use std::sync::{Arc, RwLock};

use serde_json;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::transaction::Transaction;
use uqoin_core::coin::{coin_order, coin_random, coin_symbol, 
                       coin_order_by_symbol};
use uqoin_core::state::{OrderCoinsMap, BlockInfo};

use crate::appdata::load_with_password;
use crate::api::{request_send, request_coins_map};


const MINING_CHUNK: usize = 1000000;
const BLOCK_HASH_UPDATE_TIMEOUT_MILLIS: u64 = 2000;


pub fn mining(wallet: &str, address: Option<&str>, coin: &str, 
              fee: Option<&str>, threads: usize) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Get current state
    let block_hash = request_last_block_hash(&appdata.get_validators()[0])?;
    let block_hash_arc = Arc::new(RwLock::new(block_hash));

    let coins_map = request_coins_map(wallet, &appdata.get_validators()[0])?;
    let coins_map_arc = Arc::new(RwLock::new(coins_map));

    // Loop for threads
    for _ in 0..threads {
        // Options to pass
        let schema = Schema::new();
        let miner = U256::from_hex(wallet);
        let address = address.map(|addr| U256::from_hex(addr))
                             .unwrap_or(miner.clone());
        let min_order_coin = coin_order_by_symbol(coin);
        let min_order_fee = fee.map(|s| coin_order_by_symbol(s));
        let block_hash_arc = Arc::clone(&block_hash_arc);
        let coins_map_arc = Arc::clone(&coins_map_arc);
        let wallet_key = appdata.get_wallet_key(wallet).unwrap().clone();
        let validator_root_vec = appdata.get_validators().iter().cloned()
                                        .collect::<Vec<_>>();

        // Minimum mining order
        let min_order_mining = std::cmp::min(
            min_order_coin, min_order_fee.unwrap_or(256)
        );

        // Spawn a thread
        std::thread::spawn(move || {
            // Random generator
            let mut rng = rand::rng();

            loop {
                // Clone block_hash_prev
                let block_hash_prev = block_hash_arc.read().unwrap().clone();

                // Try to mine some coins
                let coins = (0..MINING_CHUNK)
                    .map(|_| coin_random(&mut rng, &block_hash_prev, &miner))
                    .filter(|coin| coin_order(coin, &block_hash_prev, &miner) 
                                   >= min_order_mining)
                    .collect::<Vec<U256>>();

                // Try to create mining groups
                let groups: Vec<Vec<Transaction>> = 
                    if let Some(min_order_fee) = min_order_fee {
                        // Create coin-order pairs
                        let coin_order_pairs = coins.iter()
                            .map(|coin| (
                                coin.clone(), 
                                coin_order(coin, &block_hash_prev, &miner)
                            )).collect::<Vec<(U256, u64)>>();

                        // Represent as coin-fee pairs
                        let coin_fee_pairs = {
                            let mut coins_map = coins_map_arc.write().unwrap();
                            represent_as_coin_fee_pairs(coin_order_pairs, 
                                                        &mut coins_map,
                                                        min_order_coin,
                                                        min_order_fee)
                        };

                        // Collect transactions
                        coin_fee_pairs.into_iter().map(|(coin, fee)| vec![
                            Transaction::build(&mut rng, coin, address.clone(), 
                                               &wallet_key, &schema),
                            Transaction::build(&mut rng, fee, address.clone(), 
                                               &wallet_key, &schema),
                        ]).collect()
                    } else {
                        coins.iter().map(|coin| vec![
                            Transaction::build(&mut rng, coin.clone(), 
                                               address.clone(), &wallet_key, 
                                               &schema)
                        ]).collect()
                    };

                // Print just mined coins
                if !coins.is_empty() && !groups.is_empty() {
                    print!("New coins mined: ");
                    for gr in groups.iter() {
                        for tr in gr.iter() {
                            if coins.contains(&tr.coin) {
                                let order = coin_order(&tr.coin, 
                                                       &block_hash_prev, 
                                                       &miner);
                                let symbol = coin_symbol(order);
                                print!("{}-{} ", symbol, tr.coin.to_hex());
                            }
                        }
                    }
                    println!("");
                }

                // Send groups to the node
                for validator_root in validator_root_vec.iter() {
                    for group in groups.iter() {
                        let group = group.clone();
                        let validator_root = validator_root.clone();
                        std::thread::spawn(move || {
                            request_send(&group, &validator_root).unwrap();
                        });
                    }
                }
            }
        });
    }

    // Update state thread
    loop {
        std::thread::sleep(
            std::time::Duration::from_millis(BLOCK_HASH_UPDATE_TIMEOUT_MILLIS)
        );

        *block_hash_arc.write().unwrap() = 
            request_last_block_hash(&appdata.get_validators()[0])?;

        *coins_map_arc.write().unwrap() = 
            request_coins_map(wallet, &appdata.get_validators()[0])?;
    }
}


pub fn request_last_block_hash(validator_root: &str) -> std::io::Result<U256> {
    let url = format!("{}/blockchain/block-info", validator_root);
    let text = reqwest::blocking::get(url).unwrap().text().unwrap();
    let block_info: BlockInfo = serde_json::from_str(&text)?;
    Ok(block_info.hash)
}


/// Algorithm that tries to add coins from the wallet (coins_map) in order to
/// match more pairs coin-fee so the mining profit will be maximum.
pub fn represent_as_coin_fee_pairs(coin_order_pairs: Vec<(U256, u64)>,
                                   coins_map: &mut OrderCoinsMap,
                                   min_order_coin: u64,
                                   min_order_fee: u64) -> 
                                   Vec<(U256, U256)> {
    // Sizes
    let mut size = coin_order_pairs.len();
    let max_size: usize = 2 * coin_order_pairs.iter()
        .filter(|(_, or)| *or >= min_order_coin).count();

    // Sort coins by order decreasingly
    let mut coin_order_pairs = coin_order_pairs;
    coin_order_pairs.sort_by_key(|(_, order)| 255 - order);

    // Cut too cheap coins
    if size > max_size {
        coin_order_pairs.truncate(max_size);
        size = max_size;
    }
    
    if size > 0 {
        // Maximum coin order
        let order_max = coin_order_pairs[0].1;

        // Iterator for popped coins from coins_map
        let mut coins_map_copy = coins_map.clone();
        let mut coins_map_order = min_order_fee;
        let mut coins_map_iter = std::iter::from_fn(move || {
            if coins_map_copy.is_empty() {
                None
            } else {
                while (coins_map_order < order_max) && 
                      (coins_map_copy.get(&coins_map_order)
                                .map(|s| s.is_empty())
                                .unwrap_or(true)) {
                    coins_map_order += 1;
                }

                if coins_map_order >= order_max {
                    None
                } else {
                    let coin = coins_map_copy[&coins_map_order].iter()
                        .next().unwrap().clone();
                    coins_map_copy.get_mut(&coins_map_order).unwrap()
                                  .remove(&coin);
                    Some((coin, coins_map_order))
                }
            }
        });

        // If size is odd try to add one coin from the map
        if size & 1 == 1 {
            if let Some((cm, or)) = coins_map_iter.next() {
                if or < coin_order_pairs[size / 2].1 {
                    coin_order_pairs.push((cm, or));
                    size += 1;
                }
            }
        }

        // Try to add two coins from the map
        if size & 1 == 0 {
            while size < max_size {
                if let Some((cm1, or1)) = coins_map_iter.next() {
                    if let Some((cm2, or2)) = coins_map_iter.next() {
                        let or = coin_order_pairs[size / 2].1;
                        if (or > or1 + 1) && (or > or2) {
                            coin_order_pairs.push((cm1, or1));
                            coin_order_pairs.push((cm2, or2));
                            size += 2;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Remove moved coins from coins map
        for (cm, or) in coin_order_pairs.iter().rev() {
            if coins_map.contains_key(&or) {
                if !coins_map.get_mut(&or).unwrap().remove(cm) {
                    break;
                }
            }
        }

        // Return
        (0 .. size / 2).map(|i| (
            coin_order_pairs[i].0.clone(),
            coin_order_pairs[i + size / 2].0.clone(),
        )).collect()
    } else {
        vec![]
    }
}
