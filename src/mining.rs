use std::sync::RwLock;
use std::collections::HashSet;

use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::coin::{coin_order, coin_symbol, coin_mine, 
                       coin_order_by_symbol};
use uqoin_core::transaction::Transaction;
use uqoin_core::state::OrderCoinsMap;

use crate::utils::*;
use crate::try_first_validator;
use crate::api::{request_coin_info, request_send};
use crate::appdata::load_with_password;


const COINS_CACHE: &str = "~/.uqoin-client/coins.cache";


pub fn mining(wallet: &str, address: Option<&str>, fee: &str, 
              threads: usize) -> std::io::Result<()> {
    // Prepare parameters
    let miner = U256::from_hex(wallet);
    let address = U256::from_hex(address.unwrap_or(wallet));
    let min_order = coin_order_by_symbol(fee);

    // Request for appdata by password
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Get nodes and private key
    let nodes = appdata.list_validators();
    let wallet_key = appdata.get_wallet_key(wallet).unwrap();

    // Define resources
    let resource = RwLock::new(load_resource(COINS_CACHE)?);

    // Run threads in a scope
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| mine_task(&miner, min_order, &resource));
        }
        scope.spawn(|| send_task(nodes, min_order, &address,
                                 wallet_key, &resource));
    });

    Ok(())
}


fn mine_task(miner: &U256, min_order: u64, resource: &RwLock<OrderCoinsMap>) {
    // Random
    let mut rng = rand::rng();

    // Mining loop
    for coin in coin_mine(&mut rng, &miner, min_order) {
        let order = coin_order(&coin, miner);

        println!("New coin mined: {}-{}", coin_symbol(order), coin.to_hex());

        // Add the coin to local resource
        {
            let mut resource = resource.write().unwrap();
            if !resource.contains_key(&order) {
                resource.insert(order, HashSet::new());
            }
            resource.get_mut(&order).unwrap().insert(coin);
        }

        // Dump local resource to the cache file
        dump_resource(&resource.read().unwrap(), COINS_CACHE).unwrap();
    }
}


fn send_task(nodes: &[String], min_order: u64,
             address: &U256, wallet_key: &U256,
             resource: &RwLock<OrderCoinsMap>) {
    // Random
    let mut rng = rand::rng();

    // Crypto schema to sign transactions
    let schema = Schema::new();

    // Infinite loop
    loop {
        // 1. Update resource
        update_resource(nodes, resource);
        dump_resource(&resource.read().unwrap(), COINS_CACHE).unwrap();

        // 2. Prepare coins to send
        let (coins, fees) = prepare_coins(min_order, resource);

        // 3. Send transactions
        if !coins.is_empty() {
            for (coin, fee) in coins.into_iter().zip(fees.into_iter()) {
                // Build transactions
                let group = vec![
                    Transaction::build(&mut rng, coin, address.clone(), 
                                       wallet_key, 0, &schema),
                    Transaction::build(&mut rng, fee, U256::from(0), 
                                       wallet_key, 0, &schema),
                ];

                // Send transactions to all known nodes
                for node in nodes.iter() {
                    let group = group.clone();
                    let node = node.clone();
                    std::thread::spawn(move || {
                        let _ = request_send(&group, &node);
                    });
                }
            }
        }

        // Sleep for 20 seconds
        std::thread::sleep(std::time::Duration::from_millis(20000));
    }
}


fn update_resource(nodes: &[String], resource: &RwLock<OrderCoinsMap>) {
    // Collect coins to remove that have counter > 0
    let mut coins_to_remove = Vec::new();

    for coins in resource.read().unwrap().clone().values() {
        for coin in coins.iter() {
            let coin_info_res = try_first_validator!(
                nodes, request_coin_info, &coin.to_hex()
            );
            if let Some(coin_info) = coin_info_res {
                if coin_info.counter > 0 {
                    coins_to_remove.push(coin.clone());
                }
            }
        }
    }

    // Remove registered coins
    for (order, coins) in resource.write().unwrap().iter_mut() {
        for coin in coins_to_remove.iter() {
            if coins.remove(coin) {
                println!("Coin confirmed: {}-{}", 
                         coin_symbol(*order), coin.to_hex());
            }
        }
    }
}


fn prepare_coins(min_order: u64, 
                 resource: &RwLock<OrderCoinsMap>) -> (Vec<U256>, Vec<U256>) {
    let coins_map = resource.read().unwrap();

    let mut coins: Vec<U256> = coins_map.iter()
        .filter(|(order, _)| **order > min_order)
        .map(|(_, coins)| coins.iter().cloned().collect::<Vec<_>>())
        .collect::<Vec<_>>().concat();

    let mut fees: Vec<U256> = coins_map.get(&min_order)
        .unwrap_or(&HashSet::new()).iter().cloned().collect();

    let size = std::cmp::min(coins.len(), fees.len());

    coins.truncate(size);
    fees.truncate(size);

    (coins, fees)
}


fn load_resource(path: &str) -> std::io::Result<OrderCoinsMap> {
    let path = ensure_location(path)?;
    if std::fs::exists(&path)? {
        let json = std::fs::read_to_string(&path)?;
        let resource: OrderCoinsMap = serde_json::from_str(&json)?;
        Ok(resource)
    } else {
        Ok(OrderCoinsMap::new())
    }
}


fn dump_resource(resource: &OrderCoinsMap, path: &str) -> std::io::Result<()> {
    let path = ensure_location(path)?;
    let json = serde_json::to_string(resource)?;
    std::fs::write(&path, json)?;
    Ok(())
}
