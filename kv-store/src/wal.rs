use std::fs::{OpenOptions};
use std::io::{Write, Result};
use super::routes_resp::{Wal,WalOp};
// use super::ring::get_node_for_key;
use tokio::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};
use serde_json;
use chrono::Utc;
use sha2::{Sha256, Digest};
use super::config::HASH_RING;

static WAL_SEQUENCE_COUNTER: AtomicUsize = AtomicUsize::new(1); 
impl Wal {
    // Production-ready detailed format (Only format we'll use)
    pub fn to_log_line(&self) -> String {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string();
        
        // Get key from operation for node calculation
        let key = match &self.opration {
            WalOp::Set { key, .. } => key,
            WalOp::Delete { key } => key,
        };
        
        // let total_nodes = NODES.len();
        let ring=HASH_RING.read().unwrap();
       
        let node = match ring.get_node(&key) {
        Some(n) => n,
        None => {
            return "".to_string();
        }
    };
        // let node_index = get_node(&key);
        // let node_id = format!("node-{}", node_index);
        let node_id=&node.id;
        
        let operation_data = match &self.opration {
            WalOp::Set { key, value } => {
                serde_json::json!({
                    "op": "SET",
                    "key": key,
                    "value": value,
                    "key_size": key.len(),
                    "value_size": value.len()
                })
            },
            WalOp::Delete { key } => {
                serde_json::json!({
                    "op": "DELETE", 
                    "key": key,
                    "key_size": key.len()
                })
            }
        };
        
        // Calculate checksum for integrity
        let data_str = operation_data.to_string();
        let mut hasher = Sha256::new();
        hasher.update(data_str.as_bytes());
        let checksum = format!("{:x}", hasher.finalize())[..16].to_string(); // First 16 chars
        
        let log_entry = serde_json::json!({
            "seq": self.sequenceNumber,
            "timestamp": timestamp,
            "node_id": node_id,
            "operation": operation_data,
            "checksum": checksum,
            "version": "1.0"
        });
        
        format!("{}\n", log_entry.to_string())
    }
     pub fn new(opration: WalOp) -> Self {
        let seq = WAL_SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Wal {
            sequenceNumber: seq,
            opration,
            time: Instant::now(),
        }
    }
}

// Only detailed WAL logging
pub fn append_wal(entry: &Wal) -> Result<()> {
    let data = entry.to_log_line().as_bytes().to_vec();
    
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs")?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/wal_detailed.log")?;  // Better organized path
    
    file.write_all(&data)?;
    file.sync_data()?; // Ensures disk write
    
    // Performance metrics
    println!("WAL Entry Written: seq={}, size={} bytes", 
             entry.sequenceNumber, data.len());
    
    Ok(())
}

// WAL Recovery function for detailed logs
pub fn recover_from_wal(filename: &str) -> Result<Vec<serde_json::Value>> {
    use std::fs;
    use std::io::{BufRead, BufReader};
    
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        
        // Parse JSON WAL entry
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
            entries.push(entry);
        }
    }
    
    println!("Recovered {} WAL entries from {}", entries.len(), filename);
    Ok(entries)
}

// Get WAL statistics
pub fn get_wal_stats(filename: &str) -> Result<WalStats> {
    use std::fs;
    
    let metadata = fs::metadata(filename)?;
    let entries = recover_from_wal(filename)?;
    
    Ok(WalStats {
        total_entries: entries.len(),
        file_size_bytes: metadata.len(),
        last_sequence: entries.last()
            .and_then(|e| e.get("seq"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0),
    })
}

pub struct WalStats {
    pub total_entries: usize,
    pub file_size_bytes: u64,
    pub last_sequence: u64,
}