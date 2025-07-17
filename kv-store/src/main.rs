mod routes;
mod middleware;
mod routes_resp;
mod config;
mod ring;
use axum::{Router, routing::post,middleware::from_fn};
use middleware::auth_middlware;
use routes::{set_value,delete_value, get_value,login_handler};

#[tokio::main]
async fn main() {
// pub let DB=std::collections::HashMap::new();

let app = Router::new()
    .route("/set-value", post(set_value))
    .route("/get-value", post(get_value))
    .route("/delete-value", post(delete_value))
    .layer(from_fn(auth_middlware))
    .route("/login", post(login_handler));

let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(tcp_listener, app).await.unwrap();
    
}