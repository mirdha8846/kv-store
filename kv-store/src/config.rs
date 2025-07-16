use once_cell::sync::Lazy;
use sled::Db;

static NODE1_DB: Lazy<Db> = Lazy::new(|| sled::open("db/node1").unwrap());
static NODE2_DB: Lazy<Db> = Lazy::new(|| sled::open("db/node2").unwrap());
static NODE3_DB: Lazy<Db> = Lazy::new(|| sled::open("db/node3").unwrap());

pub static NODES: Lazy<Vec<&'static Db>> = Lazy::new(|| vec![&NODE1_DB, &NODE2_DB, &NODE3_DB]);
