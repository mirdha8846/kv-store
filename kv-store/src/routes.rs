// ⚡ Optional Future Upgrades

// Vector clocks	Resolve version conflicts
// Node failure detection	Auto skip dead replicas

// Gossip-based sync	Replicas fix missing data later


use metrics::{counter, histogram};
use tokio::time::Instant;
use std::env;

use chrono::{Utc,Duration};
use axum::{extract::Json, extract::State};
use dotenv::dotenv;
use crate::routes_resp::{Wal, WalOp};

use super::middleware::types;
use super::routes_resp::{SetResponse, IncomingSetRequest,
    IncomingGetRequest,GetResponse,ErrorResponse,IncomingDeleteRequest,
    DeleteResponse,LoginResponse,IncomingLoginRequest};
use super::wal::{append_wal};
use super::routes_resp::Status;
use super::config::HASH_RING;
use types::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};


pub async fn set_value(
    State(tx): State<tokio::sync::mpsc::Sender<Wal>>,
    Json(payload): Json<IncomingSetRequest>
) -> Json<SetResponse> {
    let start=Instant::now();
    counter!("route_hit",1,"route"=>"set_value");
    let key = payload.key;
    let value = payload.value;
    // let total_nodes = NODES.len();

    // Check if key exists and insert if not
    let operation_result = {
        let ring = HASH_RING.read().unwrap();
        match ring.get_node(&key) {
            Some(leader) => {
                // Check if key actually exists (not just if operation is successful)
                match leader.db.get(key.as_bytes()) {
                    Ok(Some(_)) => {
                        // Key exists, return "already present"
                        Err("Key already present")
                    },
                    Ok(None) => {
                        // Key doesn't exist, insert it
                        match leader.db.insert(key.as_bytes(), value.as_bytes()) {
                            Ok(_) => {
                                leader.db.flush().ok();
                                Ok("Key inserted successfully")
                            },
                            Err(_) => Err("Failed to insert key")
                        }
                    },
                    Err(_) => {
                        // Database error
                        Err("Database error")
                    }
                }
            },
            None => {
                return Json::from(SetResponse {
                    status: Status::Error,
                    message: "No node available".to_string(),
                });
            }
        }
    };
    
    let entry=Wal::new(WalOp::Set { key:key.clone(), value:value.clone() });
     
    match operation_result {
        Ok(_message) => {
            // Key was successfully inserted
            if let Err(e) = append_wal(&entry) {
                // Agar WAL write fail ho jaye, to safe hai request fail karna
                return Json::from(SetResponse {
                    status: Status::Error,
                    message: format!("WAL disk write failed: {}", e),
                });
            }
             
            if tx.send(entry).await.is_err() {
                eprintln!("WAL send failed");
            }
                
            let elapsed=start.elapsed().as_secs_f64();
            histogram!("request_duration_seconds",elapsed, "route" => "set_value");
            Json::from(SetResponse {
                status: Status::Success,
                message: format!("key stored"),
            })
        },
        Err(error_msg) => {
            if error_msg == "Key already present" {
                Json::from(SetResponse {
                    status: Status::Success,
                    message: format!("key already present"),
                })
            } else {
                counter!("error_count", 1, "route" => "set_value");
                Json::from(SetResponse {
                    status: Status::Error,
                    message: format!("Failed to set key '{}': {}", key, error_msg),
                })
            }
        }
    }
// }
}
// }

pub async fn get_value(Json(payload):Json<IncomingGetRequest>) -> Result<Json<GetResponse>,  Json<ErrorResponse>> {
    let start=Instant::now();
    counter!("route_hit",1,"route"=>"get_value");   
    let key = payload.key;
    
    // Try primary node first, then replicas
    let result: Result<Result<Json<GetResponse>, Json<ErrorResponse>>, &'static str> = {
        let ring = HASH_RING.read().unwrap();
        let leader = match ring.get_node(&key) {
            Some(n) => n,
            None => {
                return Err(Json::from(ErrorResponse {
                    status: Status::Error,
                    error: "No node available".to_string(),
                }));
            }
        };
        
        // Try primary node first
        match leader.db.get(key.clone().as_bytes()) {
            Ok(Some(value)) => {
                let value_str = String::from_utf8(value.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string());
                return Ok(Json::from(GetResponse {
                    status: Status::Success,
                    value: value_str,
                }));
            }
            Ok(None) => {
                // Primary node doesn't have the key, try replicas
                let all_node_ids = ring.get_follower_node_ids(&key);
                
                for replica_id in &all_node_ids {
                    if *replica_id == leader.id {
                        continue; // Skip the primary node we already tried
                    }
                    
                    // Try to get the node from ring
                    if let Some(replica_node) = ring.get_node(replica_id) {
                        match replica_node.db.get(key.clone().as_bytes()) {
                            Ok(Some(value)) => {
                                let value_str = String::from_utf8(value.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string());
                                println!("Found key '{}' in replica node '{}'", key, replica_id);
                                return Ok(Json::from(GetResponse {
                                    status: Status::Success,
                                    value: value_str,
                                }));
                            },
                            Ok(None) => continue, // Try next replica
                            Err(_) => continue,   // Node might be down, try next
                        }
                    }
                }
                
                // Key not found in any node
                Err("Key not found in any node")
            }
            Err(_e) => Err("Database error")
        }
    };
    
    // Handle the result
    match result {
        Ok(response) => response,
        Err(error_msg) => {
            let elapsed = start.elapsed().as_secs_f64();
            histogram!("request_duration_seconds", elapsed, "route" => "get_value");
            counter!("error_count", 1, "route" => "get_value");
            
            let error_response = ErrorResponse {
                status: Status::Error,
                error: error_msg.to_string(),
            };
            Err(Json::from(error_response))
        }
    }
}

pub async fn delete_value(
    State(tx): State<tokio::sync::mpsc::Sender<Wal>>,
    Json(payload): Json<IncomingDeleteRequest>
) -> Result<Json<DeleteResponse>, Json<ErrorResponse>> {
    let start=Instant::now();
    counter!("route_hit",1,"route"=>"delete_value");
    let key = payload.key;
    // Remove from Sled database
    //  let total_nodes = NODES.len();
    // let primary_index: usize = get_node_for_key(&key, total_nodes);
    let delete_result = {
        let ring = HASH_RING.read().unwrap();
        match ring.get_node(&key) {
            Some(leader) => {
                let result = leader.db.remove(key.clone().as_bytes());
                if result.is_ok() {
                    leader.db.flush().ok();
                }
                result
            },
            None => {
                return Err(Json::from(ErrorResponse {
                    status: Status::Error,
                    error: "No node available".to_string(),
                }));
            }
        }
    };
   
        if delete_result.is_ok(){
            let entry=Wal::new(WalOp::Delete { key: key });
        
          if tx.send(entry).await.is_err(){
            eprintln!("failed to delete")
          }
        let elapsed=start.elapsed().as_secs_f64();
        histogram!("request_duration_seconds",elapsed,"route"=>"delete_value");
        
              let response = DeleteResponse {
                status: Status::Success,
                message: format!("key deleted "),
            };
            Ok(Json::from(response))
        }else{
             counter!("error_count", 1, "route" => "delete_value");
             let error_response = ErrorResponse {
                    status: Status::Error,
                    error: format!("Failed to delete key: {}",key),
                };
                return Err(Json::from(error_response));
        }
   
}

pub async fn login_handler(Json(payload):Json<IncomingLoginRequest>)->Result<Json<LoginResponse>,Json<ErrorResponse>>{
    let start=Instant::now();
    counter!("route_hit",1,"route"=>"login_handler");
    dotenv().ok();
    let email=payload.email;
    let claim=Claims{
        email:email,
        exp: (Utc::now() + Duration::hours(5)).timestamp() as usize
    };
    let secret=env::var("JWT_SECRATE").unwrap();
    let token=encode(&Header::default(), &claim, &EncodingKey::from_secret(secret.as_ref()));
    match token {
        Ok(value)=>{
            let response=LoginResponse{
        status:Status::Success,
        token:value
    };
    let elapsed=start.elapsed().as_secs_f64();
    histogram!("request_duration_seconds",elapsed,"route"=>"login_handler");
      return  Ok(Json::from(response));
        },
        Err(e)=>{
            let response=ErrorResponse{
                status:Status::Error,
                error:e.to_string()
            };
             counter!("error_count", 1, "route" => "login_handler");
            return Err(Json::from(response));
        }
    }
}