use axum::{extract::Json};
use serde::de::value; 
use super::routes_resp::{SetResponse, IncomingSetRequest,
    IncomingGetRequest,GetResponse,ErrorResponse,IncomingDeleteRequest,
    DeleteResponse};
use super::config::DB;
use super::routes_resp::Status;

pub async fn set_value(Json(payload): Json<IncomingSetRequest>) -> Json<SetResponse> {
    let key = payload.key;
    let value = payload.value;

    // Check if the key already exists
    match DB.get(key.as_bytes()) {
        Ok(Some(_)) => {
            let response = SetResponse {
                status: Status::Success,
                message: "already present".to_string(),
            };
            return Json::from(response);
        }
        Ok(None) => {
            // Insert into Sled database
            if let Err(e) = DB.insert(key.as_bytes(), value.as_bytes()) {
                let response = SetResponse {
                    status: Status::Error,
                    message: format!("Failed to set key '{}': {}", key, e),
                };
                return Json::from(response);
            }
        }
        Err(e) => {
            let response = SetResponse {
                status: Status::Error,
                message: format!("Database error: {}", e),
            };
            return Json::from(response);
        }
    }

    // Flush to ensure data is persisted
    if let Err(e) = DB.flush() {
        let response = SetResponse {
            status: Status::Error,
            message: format!("Failed to flush database: {}", e),
        };
        return Json::from(response);
    }

    let response = SetResponse {
        status: Status::Success,
        message: format!("Set key '{}' with value '{}'", key, value),
    };
    Json::from(response)
}


pub async fn get_value(Json(payload):Json<IncomingGetRequest>) -> Result<Json<GetResponse>,  Json<ErrorResponse>> {
    let key = payload.key;
    
    // Get from Sled database
    match DB.get(key.clone().as_bytes()) {
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
    match DB.remove(key.clone().as_bytes()) {
        Ok(Some(_)) => {
            // Flush to ensure deletion is persisted
            if let Err(e) = DB.flush() {
                let error_response = ErrorResponse {
                    status: Status::Error,
                    error: format!("Failed to flush database: {}", e),
                };
                return Err(Json::from(error_response));
            }
            
            let response = DeleteResponse {
                status: Status::Success,
                message: format!("Deleted key '{}'", key),
            };
            Ok(Json::from(response))
        }
        Ok(None) => {
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