use std::{ time::Duration};
use tokio::sync::mpsc::{Receiver};
use crate::{config::NODES, routes_resp::{Wal, WalOp}};


pub async fn replication_worker(mut rx: Receiver<Wal>) {
    while let Some(entry) = rx.recv().await {
        match entry.opration {
            WalOp::Set { key, value } => {
                let mut success = true;
                for i in 1..NODES.len() {
                    let db = &NODES[i].db;
                    let mut retries = 3;
                    while retries > 0 {
                        if db.insert(key.as_bytes(), value.as_bytes()).is_ok() && db.flush().is_ok() {
                            break;
                        } else {
                            retries -= 1;
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                    if retries == 0 {
                        success = false;
                    }
                }
                if success {
                    println!("Set operation replicated successfully for key: {}", key);
                } else {
                    println!("Set operation failed for key: {}", key);
                }
            },
            WalOp::Delete { key } => {
                let mut success = true;
                for i in 1..NODES.len() {
                    let db = &NODES[i].db;
                    let mut retries = 3;
                    while retries > 0 {
                        if db.remove(key.as_bytes()).is_ok() && db.flush().is_ok() {
                            break;
                        } else {
                            retries -= 1;
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                    if retries == 0 {
                        success = false;
                    }
                }
                if success {
                    println!("Delete operation replicated successfully for key: {}", key);
                } else {
                    println!("Delete operation failed for key: {}", key);
                }
            }
        }
    }
}
