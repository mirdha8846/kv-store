// ⚡ Optional Future Upgrades
// Feature	Benefit
// Vector clocks	Resolve version conflicts
// Node failure detection	Auto skip dead replicas
// Async replication queue	Make writes faster
// Gossip-based sync	Replicas fix missing data later



use std::fmt::format;

use axum::{extract::Json};

use super::routes_resp::{SetResponse, IncomingSetRequest,
    IncomingGetRequest,GetResponse,ErrorResponse,IncomingDeleteRequest,
    DeleteResponse};

use super::routes_resp::Status;
use super::ring::get_node_for_key;
use super::config::NODES;

pub async fn set_value(Json(payload): Json<IncomingSetRequest>) -> Json<SetResponse> {
    let key = payload.key;
    let value = payload.value;
     let total_nodes = NODES.len();
     let mut success_count = 0;

    // todo-Check if the key already exists
    let primary_index = get_node_for_key(&key,total_nodes);
     for i in 0..3{
         let node_index = (primary_index + i) % total_nodes;
        let db = &NODES[node_index].db;
         if db.insert(key.as_bytes(), value.as_bytes()).is_ok() {
            db.flush().ok();
            success_count += 1;
              println!("count for set reached {}",success_count);
        }
     }
       if success_count >= 3 {
          println!("count reached 3 and value stored");
        let response = SetResponse {
        status: Status::Success,
        message: format!("Set key '{}' with value '{}'", key, value),
    };
    Json::from(response)

    } else {
          let response = SetResponse {
                    status: Status::Error,
                    message: format!("Failed to set key '{}'", key),
                };
                return Json::from(response);
    }
   
   
}


pub async fn get_value(Json(payload):Json<IncomingGetRequest>) -> Result<Json<GetResponse>,  Json<ErrorResponse>> {
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
            let error_response = ErrorResponse {
                status: Status::Error,
                error: format!("Key '{}' not found", key),
            };
            Err(Json::from(error_response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                status: Status::Error,
                error: format!("Database error: {}", e),
            };
            Err(Json::from(error_response))
        }
    }
}

pub async fn delete_value(Json(payload): Json<IncomingDeleteRequest>) -> Result<Json<DeleteResponse>, Json<ErrorResponse>> {
    let key = payload.key;
    
    // Remove from Sled database
     let total_nodes = NODES.len();
    let primary_index: usize = get_node_for_key(&key, total_nodes);
   
    let mut success_count=0;
    for i in 0..3{
        let node=(primary_index + i) % total_nodes;
        let db=&NODES[node].db;
        if db.remove(key.clone().as_bytes()).is_ok(){
            db.flush().ok();
            success_count+=1;
              println!("count in delete {}",success_count);
        }
        }
        if success_count>=3{
            println!("count reached 3 and deleted");
              let response = DeleteResponse {
                status: Status::Success,
                message: format!("Deleted key '{}'", key),
            };
            Ok(Json::from(response))
        }else{
             let error_response = ErrorResponse {
                    status: Status::Error,
                    error: format!("Failed to flush database: {}",key),
                };
                return Err(Json::from(error_response));
        }
   
}