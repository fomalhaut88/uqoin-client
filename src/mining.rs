use std::collections::HashSet;
use std::sync::RwLock;

use rand::prelude::*;
use uqoin_core::utils::U256;
use uqoin_core::schema::Schema;
use uqoin_core::transaction::Transaction;
use uqoin_core::coin::{coin_order, coin_order_by_symbol, coin_mine, coin_symbol};
use uqoin_core::state::OrderCoinsMap;

use crate::utils::*;
use crate::try_first_validator;
use crate::api::{request_send, request_coins_map, request_coin_info};
use crate::appdata::load_with_password;


const COINS_CACHE: &str = "~/.uqoin-client/coins.cache";


pub fn mining(wallet: &str, address: Option<&str>, coin: &str, 
              fee: &str, threads: usize) -> std::io::Result<()> {
    // Prepare parameters
    let miner = U256::from_hex(wallet);
    let address = U256::from_hex(address.unwrap_or(wallet));
    let min_order_coin = coin_order_by_symbol(coin);
    let min_order_fee = coin_order_by_symbol(fee);

    // Check orders
    assert!(min_order_coin > min_order_fee, "Fee must me smaller");

    // Request for appdata by password
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Get nodes and private key
    let nodes = appdata.list_validators();
    let wallet_key = appdata.get_wallet_key(wallet).unwrap();

    // Define resources
    let local_resource = RwLock::new(load_resource(COINS_CACHE)?);
    let remote_resource = RwLock::new(OrderCoinsMap::new());

    // Run threads in a scope
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| mine_task(&miner, min_order_fee, &local_resource));
        }
        scope.spawn(|| send_task(nodes, min_order_coin, min_order_fee, &address,
                                 wallet, wallet_key, &local_resource, 
                                 &remote_resource));
    });

    Ok(())
}


fn mine_task(miner: &U256, min_order: u64, 
             local_resource: &RwLock<OrderCoinsMap>) {
    // Random
    let mut rng = rand::rng();

    // Mining loop
    for coin in coin_mine(&mut rng, &miner, min_order) {
        let order = coin_order(&coin, miner);

        println!("New coin mined: {}-{}", coin_symbol(order), coin.to_hex());

        // Add the coin to local resource
        {
            let mut local_resource = local_resource.write().unwrap();
            if !local_resource.contains_key(&order) {
                local_resource.insert(order, HashSet::new());
            }
            local_resource.get_mut(&order).unwrap().insert(coin);
        }

        // Dump local resource to the cache file
        dump_resource(&local_resource.read().unwrap(), COINS_CACHE).unwrap();
    }
}


fn send_task(nodes: &[String], min_order_coin: u64, min_order_fee: u64, 
             address: &U256, wallet: &str, wallet_key: &U256,
             local_resource: &RwLock<OrderCoinsMap>, 
             remote_resource: &RwLock<OrderCoinsMap>) {
    // Random
    let mut rng = rand::rng();

    // Crypto schema to sign transactions
    let schema = Schema::new();

    // Infinite loop
    loop {
        // Update wallet resource
        update_remote_resource(nodes, wallet, remote_resource);

        // Update coins in local resource
        update_local_resource(local_resource, remote_resource);

        // Try to send 5 times
        for _ in 0..5 {
            // Find coin
            if let Some(coin) = get_coin(&mut rng, min_order_coin, 
                                         local_resource) {
                // Find fee
                if let Some(fee) = get_fee(&mut rng, min_order_fee, 
                                           min_order_coin,
                                           local_resource, remote_resource) {
                    println!("Coin sent: {}", coin.to_hex());

                    // Request for fee coin counter
                    let fee_counter = try_first_validator!(
                        nodes, request_coin_info, 
                        &fee.to_hex()
                    ).map(|ci| ci.counter).unwrap_or(0);

                    // Build transactions
                    let group = vec![
                        Transaction::build(&mut rng, coin, 
                                           address.clone(), 
                                           &wallet_key, 0, &schema),
                        Transaction::build(&mut rng, fee, U256::from(0), 
                                           &wallet_key, fee_counter, 
                                           &schema),
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

            // Sleep for 10 seconds
            std::thread::sleep(std::time::Duration::from_millis(10000));
        }
    }
}


fn update_remote_resource(nodes: &[String], wallet: &str, 
                          remote_resource: &RwLock<OrderCoinsMap>) {
    let coins_map_res = try_first_validator!(
        nodes, request_coins_map, wallet
    );
    if let Some(coins_map) = coins_map_res {
        if *remote_resource.read().unwrap() != coins_map {
            *remote_resource.write().unwrap() = coins_map;
        }
    }
}


fn update_local_resource(local_resource: &RwLock<OrderCoinsMap>, 
                         remote_resource: &RwLock<OrderCoinsMap>) {
    // Calculate resource difference
    let diff = {
        let mut diff: OrderCoinsMap = OrderCoinsMap::new();

        let local_resource = local_resource.read().unwrap();
        let remote_resource = remote_resource.read().unwrap();

        for (order, coins_set_local) in local_resource.iter() {
            if let Some(coins_set_remove) = remote_resource.get(order) {
                let common = coins_set_local.intersection(&coins_set_remove)
                                            .cloned()
                                            .collect::<HashSet<U256>>();
                if !common.is_empty() {
                    diff.insert(*order, common);
                }
            }
        }

        diff
    };

    for (&order, coins) in diff.iter() {
        for coin in coins.iter() {
            println!("Coin confirmed: {}-{}", coin_symbol(order), coin.to_hex());
        }
    }

    // Drop local coins if they are already in blockchain
    if !diff.is_empty() {
        let mut local_resource = local_resource.write().unwrap();

        for (order, common) in diff.iter() {
            let set = local_resource.get_mut(order).unwrap();

            if set.len() == common.len() {
                local_resource.remove(order);
            } else {
                for coin in common.iter() {
                    set.remove(coin);
                }
            }
        }
    }
}


fn get_coin<R: Rng>(rng: &mut R, min_order_coin: u64, 
                    local_resource: &RwLock<OrderCoinsMap>) -> Option<U256> {
    get_from_resource(rng, min_order_coin, 255, local_resource)
}


fn get_fee<R: Rng>(rng: &mut R, order_fee: u64, order_coin: u64,
                   local_resource: &RwLock<OrderCoinsMap>,
                   remote_resource: &RwLock<OrderCoinsMap>) -> Option<U256> {
    // Try to find coin in local resource
    if let Some(coin) = get_from_resource(rng, order_fee, order_coin - 1, 
                                          local_resource) {
        return Some(coin);
    }

    // Try to find coin in remote resource
    if let Some(coin) = get_from_resource(rng, order_fee, order_coin - 1, 
                                          remote_resource) {
        return Some(coin);
    }

    None
}


fn get_from_resource<R: Rng>(rng: &mut R, order_min: u64, order_max: u64,
                             resource: &RwLock<OrderCoinsMap>) -> Option<U256> {
    let resource = resource.read().unwrap();

    let orders_vec: Vec<u64> = resource.keys()
        .filter(|order| (**order >= order_min) && (**order <= order_max))
        .cloned().collect();

    if !orders_vec.is_empty() {
        let order: &u64 = orders_vec.choose(rng).unwrap();
        let coins_vec: Vec<&U256> = resource[order].iter().collect();
        let coin: U256 = (*coins_vec.choose(rng).unwrap()).clone();
        Some(coin)
    } else {
        None
    }
}


fn load_resource(path: &str) -> std::io::Result<OrderCoinsMap> {
    let path = ensure_location(path)?;
    if std::fs::exists(&path)? {
        let json = std::fs::read_to_string(&path)?;
        let resourse: OrderCoinsMap = serde_json::from_str(&json)?;
        Ok(resourse)
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
