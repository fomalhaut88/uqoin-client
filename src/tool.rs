use uqoin_core::utils::{U256, hash_of_u256};
use uqoin_core::schema::Schema;
use uqoin_core::seed::{Seed, Mnemonic};


pub fn gen_key() -> std::io::Result<()> {
    let schema = Schema::new();
    let mut rng = rand::rng();
    let key = schema.gen_key(&mut rng);
    println!("{}", key.to_hex());
    Ok(())
}


pub fn gen_pair() -> std::io::Result<()> {
    let schema = Schema::new();
    let mut rng = rand::rng();
    let (key, public) = schema.gen_pair(&mut rng);
    println!("{} {}", public.to_hex(), key.to_hex());
    Ok(())
}


pub fn get_public(key: &str) -> std::io::Result<()> {
    let schema = Schema::new();
    let key = U256::from_hex(key);
    let public = schema.get_public(&key);
    println!("{}", public.to_hex());
    Ok(())
}


pub fn gen_seed() -> std::io::Result<()> {
    let mut rng = rand::rng();
    let seed = Seed::random(&mut rng);
    let mnemonic = seed.mnemonic().join(" ");
    println!("{}", mnemonic);
    Ok(())
}


pub fn gen_wallets(mnemonic: &str, count: usize,
                   offset: usize) -> std::io::Result<()> {
    let mnemonic: Mnemonic = mnemonic.split(" ")
                                     .map(|s| s.to_string())
                                     .collect::<Vec<String>>()
                                     .try_into().unwrap();
    let seed = Seed::from_mnemonic(&mnemonic);
    let schema = Schema::new();
    for key in seed.gen_keys(&schema).skip(offset).take(count) {
        let public = schema.get_public(&key);
        println!("{} {}", public.to_hex(), key.to_hex());
    }
    Ok(())
}


pub fn hash(msg: &[String]) -> std::io::Result<()> {
    let unpacked: Vec<U256> = msg.iter().map(|s| U256::from_hex(s)).collect();
    let hash = hash_of_u256(unpacked.iter());
    println!("{}", hash.to_hex());
    Ok(())
}


pub fn build_signature(msg: &str, key: &str) -> std::io::Result<()> {
    let schema = Schema::new();
    let msg = U256::from_hex(msg);
    let key = U256::from_hex(key);
    let mut rng = rand::rng();
    let (sign_r, sign_s) = schema.build_signature(&mut rng, &msg, &key);
    println!("{}{}", sign_r.to_hex(), sign_s.to_hex());
    Ok(())
}


pub fn extract_public(msg: &str, signature: &str) -> std::io::Result<()> {
    let schema = Schema::new();
    let msg = U256::from_hex(msg);
    let sign_r = U256::from_hex(&signature[..64]);
    let sign_s = U256::from_hex(&signature[64..]);
    let public = schema.extract_public(&msg, &(sign_r, sign_s));
    println!("{}", public.to_hex());
    Ok(())
}
