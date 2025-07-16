use once_cell::sync::Lazy;
use sled::Db;

pub static DB: Lazy<Db> = Lazy::new(|| {
    sled::open("kv_store.db").expect("Failed to open database")
});
