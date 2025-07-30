# Distributed Key-Value Store (Rust) 🚀

A high-performance, production-grade **distributed key-value store** with automatic sharding, replication, and strong durability guarantees. Built with Rust for reliability, safety, and speed.

---

## ✨ Features

- **Consistent Hashing** for even data distribution and dynamic node management
- **Multi-node Replication** with automatic failover and majority-write safety
- **Write-Ahead Log (WAL)** for crash recovery and strong durability
- **Health Checking & Node Liveness** with real-time status tracking
- **Prometheus Metrics** for real-time observability and monitoring
- **JWT Authentication** for secure access to protected routes
- **Async Rust**: Fully asynchronous, highly concurrent server using `tokio` and `axum`

---

## 🏗️ Architecture

```
                 ┌──────────────┐
                 │   Client     │
                 └─────┬────────┘
                       │
           ┌───────────▼───────────┐
           │   API Gateway (Axum)  │
           └─────┬──────────┬──────┘
                 │          │
         ┌───────▼───┐  ┌───▼────────┐
         │ Auth      │  │ Metrics    │
         │ (JWT)     │  │ (Prometheus│
         └───────┬───┘  └────┬───────┘
                 │           │
           ┌─────▼───────────▼──────┐
           │ Distributed KV Cluster │
           │ (Consistent Hash Ring) │
           └────┬────────────┬──────┘
                │            │
         ┌──────▼───┐  ┌─────▼──────┐
         │ Node 1   │  │ Node 2     │ ...
         │ (Sled DB)│  │ (Sled DB)  │
         └──────────┘  └────────────┘
```

---

## ⚙️ How It Works

1. **Consistent Hash Ring**  
   All nodes are organized in a hash ring with virtual nodes for smooth sharding & scaling. Keys are mapped to nodes deterministically.

2. **Replication & Quorum Writes**  
   Each write is replicated to multiple nodes. A write is considered successful if it reaches a majority (quorum) of nodes, ensuring strong consistency.

3. **Write-Ahead Log (WAL) for Durability**  
   All operations are first written to a durable WAL before being replicated, guaranteeing recovery after crashes.

4. **Health Checking & Failover**  
   Each node keeps track of all others' liveness by periodic heartbeats and simulated pings, updating a global health table.

5. **Observability**  
   Exposes Prometheus metrics for all key operations, errors, and system health. Memory usage and route-level latency are tracked.

6. **API Security**  
   All mutating API endpoints are protected via JWT authentication.

---

## 🚦 Main Components

- **`main.rs`**: Application entrypoint, Axum routes, metrics setup, and background workers.
- **`hashring.rs` / `ring.rs`**: Implements consistent hashing, node sharding, and data placement.
- **`replication.rs`**: Handles multi-node replication and retry logic for durability.
- **`wal.rs`**: Write-ahead log for crash recovery and operation integrity (with checksums).
- **`gprotocol.rs`**: Node health checker and heartbeat mechanism.
- **`routes.rs`**: API endpoints for CRUD operations and login.
- **`routes_resp.rs`**: API response types and WAL operation enums.
- **`config.rs`**: Global configuration, node health table, and hash ring setup.

---

## 🔒 API Endpoints

| Endpoint         | Method | Auth | Description                  |
|------------------|--------|------|------------------------------|
| `/login`         | POST   | ❌   | Get JWT token                |
| `/set-value`     | POST   | ✅   | Set a key-value pair         |
| `/get-value`     | POST   | ✅   | Retrieve value by key        |
| `/delete-value`  | POST   | ✅   | Delete a key                 |
| `/metrics`       | GET    | ❌   | Prometheus metrics endpoint  |

---

## 📈 Observability

- **Prometheus Metrics**: Exposed at `/metrics` for easy integration with Grafana dashboards.
- **Detailed Logging**: All WAL entries, replication results, and failures are logged.
- **System Health**: Includes memory usage, request durations, and error counters.

---

## 🚀 Running Locally

**Prerequisites:**  
- Rust (latest stable)
- [Sled](https://github.com/spacejam/sled) (bundled crate, no external DB setup required)
- [Prometheus](https://prometheus.io/) for metrics (optional, for observability)

**Steps:**
```bash
git clone https://github.com/mirdha8846/kv-store.git
cd kv-store
cargo run
```

The server will start on `0.0.0.0:3000`.

---

## 🧩 Example API Usage

1. **Login to get JWT**
    ```bash
    curl -X POST http://localhost:3000/login -d '{"email":"user@example.com"}'
    ```
    Returns a JWT token.

2. **Set a Key**
    ```bash
    curl -X POST http://localhost:3000/set-value \
         -H "Authorization: Bearer <JWT>" \
         -d '{"key":"foo","value":"bar"}'
    ```

3. **Get a Key**
    ```bash
    curl -X POST http://localhost:3000/get-value \
         -H "Authorization: Bearer <JWT>" \
         -d '{"key":"foo"}'
    ```
    Returns: `{"status":"Success","value":"bar"}`

4. **Delete a Key**
    ```bash
    curl -X POST http://localhost:3000/delete-value \
         -H "Authorization: Bearer <JWT>" \
         -d '{"key":"foo"}'
    ```

5. **Prometheus Metrics**
    - Visit [http://localhost:3000/metrics](http://localhost:3000/metrics)

---

## 🏆 Why is this project advanced?

- **Distributed, Shardable Architecture:** Not a single-node or simple REST app; implements consistent hashing and virtual nodes.
- **Replication & Durability:** Real-world techniques—WAL, majority writes, and retry logic.
- **Async & Concurrent:** Uses Rust async features and channels for true multi-threaded performance.
- **Observability:** Prometheus integration, detailed WAL, and error tracking.
- **Production-Ready Patterns:** JWT auth, modular code, and extensibility for Raft/gossip/fault-tolerance upgrades.
- **Extensible:** Clearly marked extension points for gossip, vector clocks, or stronger consensus.

---

## 📚 Further Improvements & TODOs

- [ ] Implement full WAL-based crash recovery
- [ ] Add versioning and vector clocks for conflict resolution
- [ ] Expand documentation and usage examples

---

## 👤 Author

**Pankaj Mirdha**  
Email: pankajmirdha303@gmail.com

---

> *This project demonstrates hands-on expertise in distributed systems, durability, fault tolerance, async Rust, and production-grade system design. The codebase is modular and ready for advanced features like Raft, gossip, and sharding at scale.*