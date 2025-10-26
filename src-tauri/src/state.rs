use std::collections::HashMap;
use std::sync::Mutex;

/// Global state to track running game processes
pub struct RunningProcesses {
    pub processes: Mutex<HashMap<String, u32>>, // path -> process_id
}
