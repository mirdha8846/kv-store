use axum::http::{response, StatusCode};
use axum::{extract::Json}; 
use super::routes_resp::{SetResponse, IncomingSetRequest,
    IncomingGetRequest,GetResponse,ErrorResponse,IncomingDeleteRequest,
    DeleteResponse};
use super::config::DB;
use super::routes_resp::Status;

pub async fn set_value(Json(payload): Json<IncomingSetRequest>) -> Json<SetResponse> {
    let key = payload.key;
    let value = payload.value;
    
    // Lock the mutex to safely access the HashMap
    let mut db = DB.lock().unwrap();
    db.insert(key.clone(), value.clone());
    
    let response = SetResponse {
        status: Status::Success,
        message: format!("Set key '{}' with value '{}'", key, value),
    };
    Json::from(response)
}


pub async fn get_value(Json(payload):Json<IncomingGetRequest>) -> Result<Json<GetResponse>,  Json<ErrorResponse>> {
    let key = payload.key;
    
    // Lock the mutex to safely access the HashMap
    let db = DB.lock().unwrap();
    if let Some(value) = db.get(&key) {
        let response = GetResponse {
            status: Status::Success,
            value: value.clone(),
        };
        Ok(Json::from(response))
    } else {
        let error_response = ErrorResponse {
            status: Status::Error,
            error: format!("Key '{}' not found", key),
        };
        Err( Json::from(error_response))
    }
}

pub async fn delete_value(Json(payload): Json<IncomingDeleteRequest>) -> Result<Json<DeleteResponse>, Json<ErrorResponse>> {
    let key = payload.key;
    
    // Lock the mutex to safely access the HashMap
    let mut db = DB.lock().unwrap();
    if db.remove(&key).is_some() {
        let response = DeleteResponse {
            status: Status::Success,
           message: format!("Deleted key '{}'", key),
        };
        Ok(Json::from(response))
    } else {
        let error_response = ErrorResponse {
            status: Status::Error,
            error: format!("Key '{}' not found", key),
        };
        Err(Json::from(error_response))
    }
}