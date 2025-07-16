mod routes;
mod routes_resp;
mod config;
mod ring;
use axum::{Router, routing::post};
use routes::{set_value,delete_value, get_value};

#[tokio::main]
async fn main() {
// pub let DB=std::collections::HashMap::new();
let app = Router::new()
    .route("/set-value", post(set_value))
    .route("/get-value", post(get_value))
    .route("/delete-value", post(delete_value));

let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(tcp_listener, app).await.unwrap();
    
}