use uqoin_core::state::OrderCoinsMap;
use uqoin_core::transaction::Transaction;
use uqoin_core::schema::Schema;
use uqoin_core::utils::U256;
use uqoin_core::coin::coin_order_by_symbol;

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
            let total_balance: u128 = get_total_balance(&coins_map);
            if let Some(unit) = unit {
                let count = unit as i64 - 'A' as i64;
                let mut output = total_balance as f64;
                for _ in 0..count {
                    output /= 1024.0;
                }
                println!("{}", output);
            } else {
                println!("{}", total_balance);
            }
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
            &coins_map
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
            &coins_map
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
            &coins_map
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


pub fn request_send(transactions: &[Transaction], 
                    validator_root: &str) -> std::io::Result<()> {
    let url = format!("{}/client/send", validator_root);
    let client = reqwest::blocking::Client::new();
    client.post(url.clone()).json(&transactions).send()
        .map_err(|_| std::io::Error::new(
            std::io::ErrorKind::NotFound.into(), url
        ))?;
    Ok(())
}


pub fn prepare_transactions(symbol_address_pairs: &[(Option<u64>, U256)], 
                            wallet_key: &U256,
                            coins_map: &OrderCoinsMap) -> 
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
            Transaction::build(&mut rng, coin, address.clone(), wallet_key, 
                               &schema)
        }).collect();
    Ok(transactions)
}
