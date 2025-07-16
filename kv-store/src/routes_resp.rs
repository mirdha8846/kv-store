use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Status {
    Success,
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct SetResponse {
    pub status: Status,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetResponse {
    pub status: Status,
    pub value: String,
}
#[derive(Serialize, Deserialize)]
pub struct DeleteResponse {
    pub status: Status,
    pub message: String,
   
}
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: Status,
    pub error: String,
}




//Incoming request structures
#[derive(Deserialize, Serialize)]
pub struct IncomingSetRequest {
    pub key: String,
    pub value: String,
}
#[derive(Deserialize, Serialize)]
pub struct IncomingGetRequest {
    pub key: String,
}
#[derive(Deserialize, Serialize)]
pub struct IncomingDeleteRequest {
    pub key: String,
}