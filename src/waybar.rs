use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::timer::{TimerInfo, TimerState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: Option<String>,
    pub class: Option<String>,
    pub percentage: Option<u8>,
    #[serde(rename = "alt")]
    pub alt_text: Option<String>,
}

impl Default for WaybarOutput {
    fn default() -> Self {
        Self {
            text: "🍅".to_string(),
            tooltip: None,
            class: None,
            percentage: None,
            alt_text: None,
        }
    }
}

lazy_static::lazy_static! {
    static ref WAYBAR_OUTPUT: Arc<Mutex<WaybarOutput>> = Arc::new(Mutex::new(WaybarOutput::default()));
}

#[allow(dead_code)]
pub fn get_waybar_socket_path() -> Option<PathBuf> {
    let config = config::get();
    
    config.waybar_integration.socket_path.map(PathBuf::from)
}

pub fn get_waybar_output_path() -> PathBuf {
    let mut path = config::get_config_dir();
    path.push("waybar-output.json");
    path
}

pub fn format_time_remaining(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    
    format!("{:02}:{:02}", minutes, seconds)
}

pub fn update_waybar_output(timer_info: &TimerInfo) -> Result<(), String> {
    let config = config::get();
    
    if !config.waybar_integration.enabled {
        return Ok(());
    }
    
    let mut output = WaybarOutput::default();
    
    match timer_info.state {
        TimerState::Idle => {
            output.text = "🍅 Idle".to_string();
            output.class = Some("idle".to_string());
            output.tooltip = Some("Tomato Clock is idle".to_string());
        },
        TimerState::Running => {
            if let (Some(phase), Some(status)) = (&timer_info.current_phase, &timer_info.current_status) {
                let icon = phase.icon.clone().unwrap_or_else(|| "🍅".to_string());
                let status_name = &status.name;
                
                // Get time remaining or calculate it
                let time_str = if let Some(time_remaining) = timer_info.time_remaining {
                    format_time_remaining(time_remaining)
                } else {
                    // Calculate from phase duration and elapsed time
                    let total_duration = Duration::minutes(phase.duration as i64);
                    let remaining = if total_duration > timer_info.elapsed_time {
                        total_duration - timer_info.elapsed_time
                    } else {
                        Duration::zero()
                    };
                    format_time_remaining(remaining)
                };
                
                // Format according to config
                let text = config.waybar_integration.format.clone()
                    .replace("{icon}", &icon)
                    .replace("{status}", status_name)
                    .replace("{remaining}", &time_str)
                    .replace("{phase}", &phase.name);
                
                output.text = text;
                output.tooltip = Some(format!(
                    "{}: {} ({})\nRemaining: {}\nElapsed: {}",
                    status_name,
                    phase.name,
                    phase.description.clone().unwrap_or_else(|| "".to_string()),
                    time_str,
                    format_time_remaining(timer_info.elapsed_time)
                ));
                
                // Calculate percentage for progress bar
                let total_duration = Duration::minutes(phase.duration as i64);
                let percentage = if total_duration.num_seconds() > 0 {
                    let elapsed = if let Some(time_remaining) = timer_info.time_remaining {
                        total_duration - time_remaining
                    } else {
                        timer_info.elapsed_time.min(total_duration)
                    };
                    let percent = (elapsed.num_seconds() * 100) / total_duration.num_seconds();
                    Some(percent.min(100) as u8)
                } else {
                    None
                };
                
                output.percentage = percentage;
                output.class = Some("running".to_string());
                
                // Add color from phase if available
                if let Some(color) = &phase.color {
                    output.alt_text = Some(color.clone());
                }
            } else {
                output.text = "🍅 Running".to_string();
                output.class = Some("running".to_string());
            }
        },
        TimerState::Paused => {
            if let (Some(phase), Some(status)) = (&timer_info.current_phase, &timer_info.current_status) {
                let icon = phase.icon.clone().unwrap_or_else(|| "⏸️".to_string());
                let status_name = &status.name;
                
                output.text = format!("{} {} (Paused)", icon, status_name);
                output.tooltip = Some(format!(
                    "{}: {} (Paused)\nElapsed: {}",
                    status_name,
                    phase.name,
                    format_time_remaining(timer_info.elapsed_time)
                ));
                output.class = Some("paused".to_string());
            } else {
                output.text = "🍅 Paused".to_string();
                output.class = Some("paused".to_string());
            }
        },
        TimerState::Completed => {
            output.text = "🍅 Completed".to_string();
            output.class = Some("completed".to_string());
            output.tooltip = Some("Tomato Clock cycle completed".to_string());
        }
    }
    
    // Update global output
    *WAYBAR_OUTPUT.lock().unwrap() = output.clone();
    
    // Write to file for Waybar
    write_waybar_output(&output)
}

fn write_waybar_output(output: &WaybarOutput) -> Result<(), String> {
    let output_path = get_waybar_output_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create waybar output directory: {}", e))?;
        }
    }
    
    let output_str = serde_json::to_string(output)
        .map_err(|e| format!("Failed to serialize waybar output: {}", e))?;
    
    fs::write(&output_path, output_str)
        .map_err(|e| format!("Failed to write waybar output file: {}", e))?;
    
    Ok(())
}

#[allow(dead_code)]
pub fn process_waybar_click(button: u8) -> Result<(), String> {
    match button {
        1 => {
            // Left click: Start/Pause timer
            // TODO: Implement start/pause logic
            Ok(())
        },
        2 => {
            // Middle click: Stop timer
            // TODO: Implement stop logic
            Ok(())
        },
        3 => {
            // Right click: Skip current phase
            // TODO: Implement skip logic
            Ok(())
        },
        _ => Ok(()),
    }
} 