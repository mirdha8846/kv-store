mod routes;
mod middleware;
mod routes_resp;
mod config;
mod ring;
mod replication;

use sysinfo::{System};
use axum::{
    middleware::from_fn, response::IntoResponse, routing::{get, post}, Router
};
use tokio::sync::mpsc::{channel,Receiver,Sender};
use tower_http::trace::TraceLayer;
use middleware::auth_middlware;
use routes::{set_value, delete_value, get_value, login_handler};
use metrics_exporter_prometheus::{PrometheusBuilder};
use metrics::{gauge};

use crate::{replication::replication_worker, routes_resp::Wal};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let(tx,rx):(Sender<Wal>,Receiver<Wal>)=channel(100);
    tokio::spawn(replication_worker(rx));
    //todo-whole promethus setpup
    //syscall wala system
    // Build recorder 
    let recorder =
        PrometheusBuilder::new().build_recorder();
  

    // Emit a metric
     let handle = recorder.handle();
       tokio::spawn(async {
        let mut sys = System::new_all();
        loop {
            sys.refresh_memory();
            let mem = sys.used_memory() as f64; // Bytes
            gauge!("memory_usage_bytes", mem);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });


    // Clone handle for moving into the route
    let metrics_handle = handle.clone();

    // Set up Axum app
    let set_value_routes = Router::new()
        .route("/set-value", post(set_value))
        .route("/delete-value", post(delete_value))
        .with_state(tx.clone());
    
    let other_protected_routes = Router::new()
        .route("/get-value", post(get_value));
       
    
    let protected_routes = Router::new()
        .merge(set_value_routes)
        .merge(other_protected_routes)
        .layer(from_fn(auth_middlware));
    
    let app = Router::new()
        .merge(protected_routes)
        .route("/login", post(login_handler))
        .route("/metrics", get(move || async move {
           metrics_handle.render().into_response()
        }))
        .layer(TraceLayer::new_for_http());

    // Bind and run server
    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(tcp_listener, app).await.unwrap();
}
