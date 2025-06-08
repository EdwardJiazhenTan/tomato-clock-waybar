use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::status::Status;
use crate::timer::TimerState;
use crate::workflow::{Phase, Workflow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub timer_state: TimerState,
    pub current_phase: Option<Phase>,
    pub current_status: Option<Status>,
    pub current_workflow: Option<Workflow>,
    pub start_time: Option<DateTime<Local>>,
    pub elapsed_seconds: u64,
    pub last_saved: DateTime<Local>,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            timer_state: TimerState::Idle,
            current_phase: None,
            current_status: None,
            current_workflow: None,
            start_time: None,
            elapsed_seconds: 0,
            last_saved: Local::now(),
        }
    }
}

lazy_static::lazy_static! {
    static ref STATE: Arc<Mutex<PersistentState>> = Arc::new(Mutex::new(PersistentState::default()));
}

pub fn get_state_file_path() -> PathBuf {
    let mut path = config::get_config_dir();
    path.push("state.json");
    path
}

pub fn init() -> Result<(), String> {
    let state_path = get_state_file_path();
    
    // Create config directory if it doesn't exist
    if let Some(parent) = state_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create state directory: {}", e))?;
        }
    }
    
    // Load or create state file
    let state = if state_path.exists() {
        // Load existing state
        let state_str = fs::read_to_string(&state_path)
            .map_err(|e| format!("Failed to read state file: {}", e))?;
        
        serde_json::from_str::<PersistentState>(&state_str)
            .map_err(|e| format!("Failed to parse state file: {}", e))?
    } else {
        // Create default state
        let state = PersistentState::default();
        save_state(&state)?;
        state
    };
    
    // Update global state
    *STATE.lock().unwrap() = state;
    
    Ok(())
}

#[allow(dead_code)]
pub fn get() -> PersistentState {
    STATE.lock().unwrap().clone()
}

#[allow(dead_code)]
pub fn update(state: PersistentState) -> Result<(), String> {
    let mut new_state = state;
    new_state.last_saved = Local::now();
    
    *STATE.lock().unwrap() = new_state.clone();
    save_state(&new_state)
}

pub fn save_state(state: &PersistentState) -> Result<(), String> {
    let state_path = get_state_file_path();
    
    let state_str = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;
    
    fs::write(&state_path, state_str)
        .map_err(|e| format!("Failed to write state file: {}", e))?;
    
    Ok(())
} 