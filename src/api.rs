use std::sync::{Arc, RwLock};

use serde::Deserialize;
use uqoin_core::state::OrderCoinsMap;
use uqoin_core::transaction::Transaction;
use uqoin_core::schema::Schema;
use uqoin_core::utils::U256;
use uqoin_core::coin::{coin_random, coin_order};
use uqoin_core::block::Block;

use crate::utils::*;
use crate::appdata::load_with_password;


pub fn balance(wallet: &str, coins: bool, 
               detailed: bool) -> std::io::Result<()> {
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    let coins_map = request_coins_map(wallet, &appdata.get_validators()[0])?;

    if detailed {
        if coins_map.is_empty() {
            println!("Empty.");
        } else {
            for (order, coins) in coins_map.iter() {
                println!("{} - {:?}", order, coins.iter()
                    .map(|coin| coin.to_hex()).collect::<Vec<String>>());
            }
        }
    } else if coins {
        if coins_map.is_empty() {
            println!("Empty.");
        } else {
            for (order, coins) in coins_map.iter() {
                println!("{} - {}", order, coins.len());
            }
        }
    } else {
        let total_balance = get_total_balance(&coins_map);
        println!("{}", total_balance.to_decimal());
    }

    Ok(())
}


pub fn send(wallet: &str, address: &str, coin: &str, 
            fee: Option<&str>) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Request coins map
    let coins_map = request_coins_map(wallet, &appdata.get_validators()[0])?;

    // Prepare transactions
    let order = get_order_by_symbol(coin);
    let transactions = prepare_transactions(
        &[
            (Some(order), U256::from_hex(address)),
            (fee.map(|s| get_order_by_symbol(s)), U256::from(0)),
        ],
        appdata.get_wallet_key(wallet).unwrap(),
        &coins_map
    )?;

    // Request the node
    request_send(&transactions, &appdata.get_validators()[0])?;

    Ok(())
}


pub fn split(wallet: &str, coin: &str, 
             fee: Option<&str>) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Request coins map
    let coins_map = request_coins_map(wallet, &appdata.get_validators()[0])?;

    // Prepare transactions
    let order = get_order_by_symbol(coin);
    let transactions = prepare_transactions(
        &[
            (Some(order), U256::from(1)),
            (fee.map(|s| get_order_by_symbol(s)), U256::from(0)),
        ],
        appdata.get_wallet_key(wallet).unwrap(),
        &coins_map
    )?;

    // Request the node
    request_send(&transactions, &appdata.get_validators()[0])?;

    Ok(())
}


pub fn merge(wallet: &str, coin: &str, 
             fee: Option<&str>) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Request coins map
    let coins_map = request_coins_map(wallet, &appdata.get_validators()[0])?;

    // Prepare transactions
    let order = get_order_by_symbol(coin);
    let transactions = prepare_transactions(
        &[
            (Some(order - 1), U256::from(2)),
            (Some(order - 2), U256::from(2)),
            (Some(order - 2), U256::from(2)),
            (fee.map(|s| get_order_by_symbol(s)), U256::from(0)),
        ],
        appdata.get_wallet_key(wallet).unwrap(),
        &coins_map
    )?;

    // Request the node
    request_send(&transactions, &appdata.get_validators()[0])?;

    Ok(())
}


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
struct BlockData {
    bix: u64,
    block: Block,
    transactions: Option<Vec<Transaction>>,
}


fn request_coins_map(wallet: &str, 
                     validator_root: &str) -> std::io::Result<OrderCoinsMap> {
    let url = format!("{}/client/coins?wallet={}", validator_root, wallet);
    let text = reqwest::blocking::get(url).unwrap().text().unwrap();
    Ok(serde_json::from_str(&text)?)
}


fn request_send(transactions: &[Transaction], 
                validator_root: &str) -> std::io::Result<()> {
    let url = format!("{}/client/send", validator_root);
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url).json(&transactions).send().unwrap();
    if resp.status() == 200 {
        println!("Transfer reqwested successfully.");
    } else {
        println!("Something is wrong");
    };
    Ok(())
}


fn request_last_block(validator_root: &str) -> std::io::Result<Block> {
    let url = format!("{}/blockchain/block", validator_root);
    let text = reqwest::blocking::get(url).unwrap().text().unwrap();
    let data: BlockData = serde_json::from_str(&text)?;
    Ok(data.block)
}


fn prepare_transactions(symbol_address_pairs: &[(Option<u64>, U256)], 
                        wallet_key: &U256,
                        coins_map: &OrderCoinsMap) -> 
                        std::io::Result<Vec<Transaction>> {
    let mut rng = rand::rng();
    let schema = Schema::new();
    let transactions = symbol_address_pairs.iter()
        .filter(|(order, _)| order.is_some())
        .map(|(order, address)| {
            let coin_set = coins_map.get(&order.unwrap()).unwrap();
            let coin: U256 = coin_set.iter().next().unwrap().clone();
            Transaction::build(&mut rng, coin, address.clone(), wallet_key, 
                               &schema)
        }).collect();
    Ok(transactions)
}
