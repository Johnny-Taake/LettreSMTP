use std::collections::HashMap;
use tokio::sync::Mutex;

pub struct AppState {
    // NOTE: IP -> timestamps
    pub flood_control: Mutex<HashMap<String, Vec<i64>>>,
}
