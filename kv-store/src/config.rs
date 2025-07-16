use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static DB: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});
