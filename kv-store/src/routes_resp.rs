use serde::{Deserialize, Serialize};
use tokio::time::Instant;

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
#[derive(Deserialize, Serialize)]
pub struct LoginResponse{
    pub status:Status,
    pub token:String
}
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: Status,
    pub error: String,
}

#[derive(Clone, Debug)]
pub enum WalOp {
    Set { key: String, value: String },
    Delete { key: String },
}




//Incoming request structures
#[derive(Clone,Debug)]
//wal->write ahead log
pub struct Wal{
pub opration:WalOp,
pub time:Instant
}
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
#[derive(Deserialize, Serialize)]
pub struct IncomingLoginRequest{
    pub email:String
}