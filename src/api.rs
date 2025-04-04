use std::collections::HashSet;

use uqoin_core::state::{OrderCoinsMap, CoinInfo};
use uqoin_core::transaction::Transaction;
use uqoin_core::schema::Schema;
use uqoin_core::utils::U256;
use uqoin_core::coin::{coin_order_by_symbol, coin_symbol};

use crate::{try_first_validator, try_all_validators};
use crate::utils::*;
use crate::appdata::load_with_password;


pub fn balance(wallet: &str, coins: bool, 
               detailed: bool, unit: Option<char>) -> std::io::Result<()> {
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    if let Some(coins_map) = try_first_validator!(appdata.list_validators(), 
                                                  request_coins_map, wallet) {
        if detailed {
            if coins_map.is_empty() {
                println!("Empty.");
            } else {
                // Convert coins map to order-coins pairs
                let mut order_coins_vec = coins_map.into_iter()
                    .collect::<Vec<(u64, HashSet<U256>)>>();

                // Sort pairs by order descendingly
                order_coins_vec.sort_by(|a, b| b.0.cmp(&a.0));

                // Print order-coin pairs
                for (order, coins) in order_coins_vec.into_iter() {
                    for coin in coins.into_iter() {
                        println!("{}\t{}", coin_symbol(order), coin.to_hex());
                    }
                }
            }
        } else if coins {
            if coins_map.is_empty() {
                println!("Empty.");
            } else {
                // Convert coins map to order-count pairs
                let mut order_count_vec = coins_map.into_iter()
                    .map(|(order, coins)| (order, coins.len()))
                    .collect::<Vec<(u64, usize)>>();

                // Sort pairs by order descendingly
                order_count_vec.sort_by(|a, b| b.0.cmp(&a.0));

                // Print order-count pairs
                for (order, count) in order_count_vec.into_iter() {
                    println!("{}\t{}", coin_symbol(order), count);
                }
            }
        } else {
            // Get unit
            let unit = unit.unwrap_or('A');

            // Convert the balance into the units
            let mut output = get_total_balance(&coins_map) as f64;
            for _ in 0 .. (unit as usize - 'A' as usize) {
                output /= 1024.0;
            }

            // Print the balance
            println!("{} {}", output, unit);
        }
    } else {
        println!("Error: cound not reach validators.");
    }

    Ok(())
}


pub fn send(wallet: &str, address: &str, coin: &str, 
            fee: Option<&str>) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Request coins map
    if let Some(coins_map) = try_first_validator!(appdata.list_validators(), 
                                                  request_coins_map, wallet) {

        // Prepare transactions
        let order = coin_order_by_symbol(coin);
        let transactions = prepare_transactions(
            &[
                (Some(order), U256::from_hex(address)),
                (fee.map(|s| coin_order_by_symbol(s)), U256::from(0)),
            ],
            appdata.get_wallet_key(wallet).unwrap(),
            &coins_map, appdata.list_validators()
        )?;

        // Request the node
        if try_all_validators!(appdata.list_validators(), request_send, 
                               &transactions) > 0 {
            println!("Transaction request has been sent.");
        } else {
            println!("Error sending transaction request.");
        }
    } else {
        println!("Error: cound not reach validators.");
    }

    Ok(())
}


pub fn split(wallet: &str, coin: &str, 
             fee: Option<&str>) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Request coins map
    if let Some(coins_map) = try_first_validator!(appdata.list_validators(), 
                                                  request_coins_map, wallet) {
        // Prepare transactions
        let order = coin_order_by_symbol(coin);
        let transactions = prepare_transactions(
            &[
                (Some(order), U256::from(1)),
                (fee.map(|s| coin_order_by_symbol(s)), U256::from(0)),
            ],
            appdata.get_wallet_key(wallet).unwrap(),
            &coins_map, appdata.list_validators()
        )?;

        // Request the node
        if try_all_validators!(appdata.list_validators(), request_send, 
                               &transactions) > 0 {
            println!("Transaction request has been sent.");
        } else {
            println!("Error sending transaction request.");
        }
    } else {
        println!("Error: cound not reach validators.");
    }

    Ok(())
}


pub fn merge(wallet: &str, coin: &str, 
             fee: Option<&str>) -> std::io::Result<()> {
    // Load AppData
    let appdata = load_with_password()?.0;
    appdata.check_not_empty()?;

    // Request coins map
    if let Some(coins_map) = try_first_validator!(appdata.list_validators(), 
                                                  request_coins_map, wallet) {
        // Prepare transactions
        let order = coin_order_by_symbol(coin);
        let transactions = prepare_transactions(
            &[
                (Some(order - 1), U256::from(2)),
                (Some(order - 2), U256::from(2)),
                (Some(order - 2), U256::from(2)),
                (fee.map(|s| coin_order_by_symbol(s)), U256::from(0)),
            ],
            appdata.get_wallet_key(wallet).unwrap(),
            &coins_map, appdata.list_validators()
        )?;

        // Request the node
        if try_all_validators!(appdata.list_validators(), request_send, 
                               &transactions) > 0 {
            println!("Transaction request has been sent.");
        } else {
            println!("Error sending transaction request.");
        }
    } else {
        println!("Error: cound not reach validators.");
    }

    Ok(())
}


pub fn request_coins_map(wallet: &str, 
                         validator_root: &str) -> 
                         std::io::Result<OrderCoinsMap> {
    let url = format!("{}/client/coins?wallet={}", validator_root, wallet);
    let resp = reqwest::blocking::get(url.clone())
        .map_err(|_| std::io::Error::new(
            std::io::ErrorKind::NotFound.into(), url
        ))?;
    let text = resp.text().unwrap();
    Ok(serde_json::from_str(&text)?)
}


pub fn request_coin_info(coin: &str, 
                         validator_root: &str) -> 
                         std::io::Result<CoinInfo> {
    let url = format!("{}/coin/info?coin={}", validator_root, coin);
    let resp = reqwest::blocking::get(url.clone())
        .map_err(|_| std::io::Error::new(
            std::io::ErrorKind::NotFound.into(), url
        ))?;
    let text = resp.text().unwrap();
    Ok(serde_json::from_str(&text)?)
}


pub fn request_send(transactions: &[Transaction], 
                    validator_root: &str) -> std::io::Result<()> {
    let url = format!("{}/client/send", validator_root);
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url.clone()).json(&transactions).send()
        .map_err(|_| std::io::Error::new(
            std::io::ErrorKind::NotFound.into(), url
        ))?;
    if resp.status() != 200 {
        let text = resp.text().unwrap_or("".to_string());
        if text.is_empty() {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "node error"))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, text))
        }
    } else {
        Ok(())
    }
}


pub fn prepare_transactions(symbol_address_pairs: &[(Option<u64>, U256)], 
                            wallet_key: &U256,
                            coins_map: &OrderCoinsMap,
                            validators: &[String]) -> 
                            std::io::Result<Vec<Transaction>> {
    let mut rng = rand::rng();
    let schema = Schema::new();
    let mut coins_map_copy = coins_map.clone();
    let transactions = symbol_address_pairs.iter()
        .filter(|(order, _)| order.is_some())
        .map(|(order, address)| {
            let coin_set = coins_map_copy.get_mut(&order.unwrap()).unwrap();
            let coin: U256 = coin_set.iter().next().unwrap().clone();
            coin_set.remove(&coin);
            let coin_info = try_first_validator!(validators, request_coin_info, 
                                                 &coin.to_hex()).unwrap();
            Transaction::build(&mut rng, coin, address.clone(), wallet_key, 
                               coin_info.counter, &schema)
        }).collect();
    Ok(transactions)
}
