use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Mutex,
};

use lazy_static::lazy_static;
use local_ip_address::local_ip;
use snowflake::SnowflakeIdGenerator;

lazy_static! {
    pub static ref SNOWFLAKE: Mutex<SnowflakeIdGenerator> = Mutex::new(initialize_snowflake_id());
}

fn initialize_snowflake_id() -> SnowflakeIdGenerator {
    let ip = local_ip().unwrap().to_string();
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    let ip = hasher.finish() as i32;
    SnowflakeIdGenerator::new(1, ip)
}
