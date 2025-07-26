
use super::config::{NodeHealth, HEALTH_TABLE,HASH_RING}; // static health table
use std::time::Duration;

pub async fn start_local_health_checker(_my_id: String) {
    loop {
        {
            let ring = HASH_RING.read().unwrap();
            let all_ids: Vec<_> = ring.get_all_node_ids(); // e.g., ["node0", "node1", ...]

            for node_id in all_ids {
                // simulate a ping by doing a dummy read
                let result = ring.get_node_by_id(&node_id)
                    .and_then(|node| node.db.get(b"dummy").ok());

                let alive = result.is_some();

                let mut health_table = HEALTH_TABLE.write().unwrap();
                let now = current_timestamp_ms();

                health_table.insert(node_id.clone(), NodeHealth {
                    id: node_id.clone(),
                    last_heartbeat: now,
                    is_alive: alive,
                });
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn start_heartbeat_updater(my_id: String) {
    loop {
        let now = current_timestamp_ms();

        {
            let mut table = HEALTH_TABLE.write().unwrap();
            table.insert(my_id.clone(), NodeHealth {
                id: my_id.clone(),
                last_heartbeat: now,
                is_alive: true,
            });
        } // Lock guard is dropped here
        
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
