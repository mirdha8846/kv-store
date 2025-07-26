use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use crate::{config::HASH_RING, routes_resp::{Wal, WalOp}};

pub async fn replication_worker(mut rx: Receiver<Wal>) {
    while let Some(entry) = rx.recv().await {
        match &entry.opration {
            WalOp::Set { key, value } => {
                // Get node IDs first
                let all_node_ids = {
                    let ring = HASH_RING.read().unwrap();
                    ring.get_follower_node_ids(&key)
                };
                
                let mut success_count = 0;
                let mut total_attempts = 0;
                
                // Replicate to all nodes
                for node_id in &all_node_ids {
                    total_attempts += 1;
                    let mut retries = 3;
                    
                    while retries > 0 {
                        // Get node and perform operation within lock scope
                        let operation_result = {
                            let ring = HASH_RING.read().unwrap();
                            if let Some(node) = ring.get_node(node_id) {
                                let result = node.db.insert(key.as_bytes(), value.as_bytes());
                                if result.is_ok() {
                                    node.db.flush().is_ok()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };
                        
                        if operation_result {
                            success_count += 1;
                            println!("Set operation replicated to node: {}", node_id);
                            break;
                        } else {
                            println!("Replication failed for node {}", node_id);
                        }
                        
                        retries -= 1;
                        if retries > 0 {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                    
                    if retries == 0 {
                        println!("Failed to replicate SET to node {} after 3 retries", node_id);
                    }
                }
                
                let success_ratio = success_count as f32 / total_attempts as f32;
                if success_ratio >= 0.5 { // Majority success
                    println!("Set operation replicated successfully for key: {} ({}/{} nodes)", 
                            key, success_count, total_attempts);
                } else {
                    println!("Set operation failed for key: {} ({}/{} nodes)", 
                            key, success_count, total_attempts);
                }
            },
            
            WalOp::Delete { key } => {
                // Get node IDs first
                let all_node_ids = {
                    let ring = HASH_RING.read().unwrap();
                    ring.get_follower_node_ids(&key)
                };
                
                let mut success_count = 0;
                let mut total_attempts = 0;
                
                // Replicate delete to all nodes
                for node_id in &all_node_ids {
                    total_attempts += 1;
                    let mut retries = 3;
                    
                    while retries > 0 {
                        // Get node and perform operation within lock scope
                        let operation_result = {
                            let ring = HASH_RING.read().unwrap();
                            if let Some(node) = ring.get_node(node_id) {
                                let result = node.db.remove(key.as_bytes());
                                if result.is_ok() {
                                    node.db.flush().is_ok()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };
                        
                        if operation_result {
                            success_count += 1;
                            println!("Delete operation replicated to node: {}", node_id);
                            break;
                        } else {
                            println!("Delete replication failed for node {}", node_id);
                        }
                        
                        retries -= 1;
                        if retries > 0 {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                    
                    if retries == 0 {
                        println!("Failed to replicate DELETE to node {} after 3 retries", node_id);
                    }
                }
                
                let success_ratio = success_count as f32 / total_attempts as f32;
                if success_ratio >= 0.5 { // Majority success
                    println!("Delete operation replicated successfully for key: {} ({}/{} nodes)", 
                            key, success_count, total_attempts);
                } else {
                    println!("Delete operation failed for key: {} ({}/{} nodes)", 
                            key, success_count, total_attempts);
                }
            }
        }
    }
}
