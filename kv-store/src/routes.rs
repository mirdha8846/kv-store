// ⚡ Optional Future Upgrades
// Feature	Benefit
// Vector clocks	Resolve version conflicts
// Node failure detection	Auto skip dead replicas
// Async replication queue	Make writes faster
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
use super::ring::get_node_for_key;
use super::config::NODES;
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
    let total_nodes = NODES.len();

    //todo->check same key already present or not
    
    let entry=Wal::new(WalOp::Set { key:key.clone(), value:value.clone() });
     
    
    
    let primary_index = get_node_for_key(&key,total_nodes);
     
        let db = &NODES[primary_index].db;

        if db.get(key.as_bytes()).is_ok(){
           
            let response = SetResponse {
                    status: Status::Success,
                    message: format!("key already present"),
                };
               return  Json::from(response);
        }

         if let Err(e) = append_wal(&entry) {
        // Agar WAL write fail ho jaye, to safe hai request fail karna
        return Json::from(SetResponse {
            status: Status::Error,
            message: format!("WAL disk write failed: {}", e),
        });
    }
         
         if db.insert(key.as_bytes(), value.as_bytes()).is_ok() {
            db.flush().ok();
         
             if tx.send(entry).await.is_err() {
            eprintln!("WAL send failed");
        }
            
            let elapsed=start.elapsed().as_secs_f64();
            histogram!("request_duration_seconds",elapsed, "route" => "set_value");
             let response = SetResponse {
                    status: Status::Success,
                    message: format!("key stored"),
                };
                Json::from(response)
        }
     else {
         counter!("error_count", 1, "route" => "set_value");
          let response = SetResponse {
                    status: Status::Error,
                    message: format!("Failed to set key '{}'", key),
                };
                return Json::from(response);
    }
}

pub async fn get_value(Json(payload):Json<IncomingGetRequest>) -> Result<Json<GetResponse>,  Json<ErrorResponse>> {
    let start=Instant::now();
    counter!("route_hit",1,"route"=>"get_value");   
    let key = payload.key;
    let total_nodes = NODES.len();
    let primary_index = get_node_for_key(&key, total_nodes);
    let db = &NODES[primary_index].db;
    match db.get(key.clone().as_bytes()) {
        Ok(Some(value)) => {
            // Convert bytes back to string
            let value_str = String::from_utf8(value.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string());
            let response = GetResponse {
                status: Status::Success,
                value: value_str,
            };
            Ok(Json::from(response))
        }
        Ok(None) => {
            
            for i in 1..3{
                let node_index = (primary_index + i) % total_nodes;
                let db_replica = &NODES[node_index].db;
                if db_replica.get(key.clone().as_bytes()).is_ok(){
                    match db_replica.get(key.clone().as_bytes()){
                        Ok(Some(value))=>{
                            let value_str=String::from_utf8(value.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string());
                            let response = GetResponse {
                                status: Status::Success,
                                value: value_str,
                            };
                            return Ok(Json::from(response));
                        },
                        Ok(None)=>{
                            continue;
                        },
                        Err(_)=>{
                            continue;
                        }

                    }
                }
            }
                let elapsed=start.elapsed().as_secs_f64();
                histogram!("request_duration_seconds", elapsed,"route"=>"get_value");
                
            let error_response = ErrorResponse {
                status: Status::Error,
                error: format!("Key '{}' not found", key),
            };
             counter!("error_count", 1, "route" => "get_value");
            Err(Json::from(error_response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                status: Status::Error,
                error: format!("Database error: {}", e),
            };
             counter!("error_count", 1, "route" => "get_value");
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
     let total_nodes = NODES.len();
    let primary_index: usize = get_node_for_key(&key, total_nodes);
   
   
        let db=&NODES[primary_index].db;
        if db.remove(key.clone().as_bytes()).is_ok(){
            db.flush().ok();
            let entry=Wal::new(WalOp::Delete { key: key });
        
          if tx.send(entry).await.is_err(){
            eprintln!("failed to delete")
          }
        let elapsed=start.elapsed().as_secs_f64();
        histogram!("request_duration_seconds",elapsed,"route"=>"delete_value");
        
             counter!("error_count", 1, "route" => "delete_value");
           
              let response = DeleteResponse {
                status: Status::Success,
                message: format!("key deleted "),
            };
            Ok(Json::from(response))
        }else{
             counter!("error_count", 1, "route" => "delete_value");
             let error_response = ErrorResponse {
                    status: Status::Error,
                    error: format!("Failed to flush database: {}",key),
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