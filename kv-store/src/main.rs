mod routes;
mod middleware;
mod routes_resp;
mod config;
mod ring;

use sysinfo::{System};
use axum::{
    Router,
    routing::{get, post},
    middleware::from_fn,
    response::IntoResponse,
};
use middleware::auth_middlware;
use routes::{set_value, delete_value, get_value, login_handler};
use metrics_exporter_prometheus::{PrometheusBuilder};
use metrics::{gauge};

#[tokio::main]
async fn main() {
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
    let app = Router::new()
        .route("/set-value", post(set_value))
        .route("/get-value", post(get_value))
        .route("/delete-value", post(delete_value))
        .layer(from_fn(auth_middlware))
        .route("/login", post(login_handler))
        .route("/metrics", get(move || async move {
           metrics_handle.render().into_response()
        }));

    // Bind and run server
    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(tcp_listener, app).await.unwrap();
}
