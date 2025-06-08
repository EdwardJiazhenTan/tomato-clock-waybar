use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time;

use crate::status::Status;
use crate::workflow::{Phase, Workflow};
use crate::persistence;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerInfo {
    pub state: TimerState,
    pub current_phase: Option<Phase>,
    pub time_remaining: Option<Duration>,
    pub elapsed_time: Duration,
    pub current_status: Option<Status>,
    pub current_workflow: Option<Workflow>,
    pub start_time: Option<DateTime<Local>>,
    pub pause_time: Option<DateTime<Local>>,
}

impl Default for TimerInfo {
    fn default() -> Self {
        Self {
            state: TimerState::Idle,
            current_phase: None,
            time_remaining: None,
            elapsed_time: Duration::zero(),
            current_status: None,
            current_workflow: None,
            start_time: None,
            pause_time: None,
        }
    }
}

#[derive(Debug)]
pub enum TimerCommand {
    Start {
        workflow: Option<Workflow>,
        status: Option<Status>,
    },
    Pause,
    Resume,
    Stop,
    Skip,
}

#[derive(Debug)]
pub enum TimerEvent {
    Started {
        #[allow(dead_code)]
        workflow: Workflow,
        #[allow(dead_code)]
        status: Status,
    },
    PhaseChanged {
        #[allow(dead_code)]
        phase: Phase,
    },
    Paused,
    Resumed,
    Stopped,
    Completed,
}

pub struct Timer {
    info: Arc<Mutex<TimerInfo>>,
    command_tx: mpsc::Sender<TimerCommand>,
    // Keep a channel for events but mark it as unused to suppress warnings
    #[allow(dead_code)]
    event_rx: mpsc::Receiver<TimerEvent>,
}

impl Timer {
    pub async fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);
        
        // Try to load persisted state
        let persisted_state = persistence::get();
        
        // Create initial timer info from persisted state
        let mut timer_info = TimerInfo {
            state: persisted_state.timer_state,
            current_phase: persisted_state.current_phase.clone(),
            time_remaining: None, // We'll recalculate this if needed
            elapsed_time: Duration::seconds(persisted_state.elapsed_seconds as i64),
            current_status: persisted_state.current_status.clone(),
            current_workflow: persisted_state.current_workflow.clone(),
            start_time: persisted_state.start_time,
            pause_time: None, // We don't persist pause time
        };
        
        // Calculate time_remaining based on current phase and elapsed time
        if timer_info.state == TimerState::Running && timer_info.current_phase.is_some() {
            let phase = timer_info.current_phase.as_ref().unwrap();
            let total_duration = Duration::minutes(phase.duration as i64);
            let elapsed = timer_info.elapsed_time;
            
            if elapsed < total_duration {
                timer_info.time_remaining = Some(total_duration - elapsed);
            } else {
                // Phase should have been completed
                timer_info.time_remaining = Some(Duration::zero());
            }
        }
        
        let info = Arc::new(Mutex::new(timer_info));
        
        // Spawn timer logic task with a cloned event sender
        let timer_info_clone = Arc::clone(&info);
        
        tokio::spawn(async move {
            timer_logic_task(timer_info_clone, command_rx, event_tx).await;
        });
        
        // Spawn a task to consume events so they don't pile up
        tokio::spawn(async move {
            event_consumer_task(event_rx).await;
        });
        
        Timer {
            info: Arc::clone(&info),
            command_tx,
            event_rx: mpsc::channel(100).1,  // Create a dummy receiver
        }
    }
    
    pub fn get_info(&self) -> TimerInfo {
        self.info.lock().unwrap().clone()
    }
    
    pub async fn send_command(&self, command: TimerCommand) -> Result<(), &'static str> {
        self.command_tx.send(command).await.map_err(|_| "Failed to send command")
    }
    
    // Keep this method for future use but suppress warnings
    #[allow(dead_code)]
    pub async fn receive_event(&mut self) -> Option<TimerEvent> {
        self.event_rx.recv().await
    }
}

async fn timer_logic_task(
    timer_info: Arc<Mutex<TimerInfo>>,
    mut command_rx: mpsc::Receiver<TimerCommand>,
    event_tx: mpsc::Sender<TimerEvent>,
) {
    let mut interval = time::interval(time::Duration::from_secs(1));
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Update timer if running
                let mut update_needed = false;
                {
                    let mut info = timer_info.lock().unwrap();
                    if info.state == TimerState::Running {
                        if let Some(mut remaining) = info.time_remaining {
                            // Decrease remaining time
                            if remaining > Duration::seconds(1) {
                                remaining = remaining - Duration::seconds(1);
                                info.time_remaining = Some(remaining);
                                info.elapsed_time = info.elapsed_time + Duration::seconds(1);
                                
                                // Save state every 10 seconds to avoid too frequent writes
                                if info.elapsed_time.num_seconds() % 10 == 0 {
                                    save_timer_state(&info);
                                }
                            } else {
                                // Phase completed
                                info.time_remaining = None;
                                update_needed = true;
                                
                                // Save state on phase completion
                                save_timer_state(&info);
                            }
                        }
                    }
                }
                
                if update_needed {
                    // Handle phase transition logic here
                    let phase_completed = {
                        let mut info = timer_info.lock().unwrap();
                        let workflow_opt = info.current_workflow.clone();
                        let current_phase_opt = info.current_phase.clone();
                        
                        if let (Some(workflow), Some(current_phase)) = (workflow_opt, current_phase_opt) {
                            // Find the current phase index
                            if let Some(current_index) = workflow.phases.iter().position(|p| p.name == current_phase.name) {
                                // Check if there are more phases
                                if current_index + 1 < workflow.phases.len() {
                                    // Move to the next phase
                                    let next_phase = workflow.phases[current_index + 1].clone();
                                    info.current_phase = Some(next_phase.clone());
                                    info.time_remaining = Some(Duration::minutes(next_phase.duration as i64));
                                    info.elapsed_time = Duration::zero();
                                    
                                    // Save state after phase transition
                                    save_timer_state(&info);
                                    
                                    // Return the phase for the event
                                    next_phase
                                } else if workflow.repeatable {
                                    // If workflow is repeatable, start over
                                    let next_phase = workflow.phases[0].clone();
                                    info.current_phase = Some(next_phase.clone());
                                    info.time_remaining = Some(Duration::minutes(next_phase.duration as i64));
                                    info.elapsed_time = Duration::zero();
                                    
                                    // Save state after phase transition
                                    save_timer_state(&info);
                                    
                                    // Return the phase for the event
                                    next_phase
                                } else {
                                    // End of workflow
                                    info.state = TimerState::Completed;
                                    info.current_phase = None;
                                    info.time_remaining = None;
                                    
                                    // Save state after completion
                                    save_timer_state(&info);
                                    
                                    return;
                                }
                            } else {
                                // This shouldn't happen, but just in case
                                info.state = TimerState::Idle;
                                info.current_phase = None;
                                info.time_remaining = None;
                                
                                // Save state after reset
                                save_timer_state(&info);
                                
                                return;
                            }
                        } else {
                            // No workflow or phase
                            info.state = TimerState::Idle;
                            
                            // Save state after reset
                            save_timer_state(&info);
                            
                            return;
                        }
                    };
                    
                    // Send phase changed event after releasing the lock
                    let send_result = event_tx.send(TimerEvent::PhaseChanged {
                        phase: phase_completed,
                    }).await;
                    if send_result.is_err() {
                        println!("Failed to send phase changed event");
                    }
                }
            }
            
            Some(command) = command_rx.recv() => {
                match command {
                    TimerCommand::Start { workflow, status } => {
                        // Start timer logic
                        let event = {
                            // Create local variables before we take the lock
                            let workflow_to_use = workflow.unwrap_or_else(|| {
                                // TODO: Get default workflow from config
                                Workflow::default()
                            });
                            
                            let status_to_use = status.unwrap_or_else(|| {
                                // TODO: Get default status from config
                                Status::default()
                            });
                            
                            // Prepare the initial phase if there is one
                            let initial_phase = workflow_to_use.phases.first().cloned();
                            
                            // Now take the lock and update
                            let mut info = timer_info.lock().unwrap();
                            
                            // Set initial phase
                            if let Some(phase) = &initial_phase {
                                info.current_phase = Some(phase.clone());
                                info.time_remaining = Some(Duration::minutes(phase.duration as i64));
                            }
                            
                            info.current_workflow = Some(workflow_to_use.clone());
                            info.current_status = Some(status_to_use.clone());
                            info.state = TimerState::Running;
                            info.start_time = Some(Local::now());
                            info.elapsed_time = Duration::zero();
                            
                            // Save state after starting
                            save_timer_state(&info);
                            
                            // Prepare the event to send after we release the lock
                            TimerEvent::Started {
                                workflow: workflow_to_use,
                                status: status_to_use,
                            }
                        };
                        
                        // Send event after releasing the lock
                        let send_result = event_tx.send(event).await;
                        if send_result.is_err() {
                            println!("Failed to send start event");
                        }
                    }
                    
                    TimerCommand::Pause => {
                        // We'll prepare the event outside the lock
                        let should_pause;
                        let mut paused_info = None;
                        {
                            let mut info = timer_info.lock().unwrap();
                            should_pause = info.state == TimerState::Running;
                            if should_pause {
                                info.state = TimerState::Paused;
                                info.pause_time = Some(Local::now());
                                
                                // Save state after pausing
                                save_timer_state(&info);
                                
                                // Clone the info for use outside the lock
                                paused_info = Some(info.clone());
                            }
                        }
                        
                        // Only send event if we actually paused
                        if should_pause {
                            // Ensure the state is properly persisted
                            if let Some(info) = paused_info {
                                *timer_info.lock().unwrap() = info;
                            }
                            
                            let send_result = event_tx.send(TimerEvent::Paused).await;
                            if send_result.is_err() {
                                println!("Failed to send pause event");
                            }
                        }
                    }
                    
                    TimerCommand::Resume => {
                        // We'll prepare the event outside the lock
                        let should_resume;
                        let mut resumed_info = None;
                        {
                            let mut info = timer_info.lock().unwrap();
                            should_resume = info.state == TimerState::Paused;
                            if should_resume {
                                info.state = TimerState::Running;
                                info.pause_time = None;
                                
                                // Save state after resuming
                                save_timer_state(&info);
                                
                                // Clone the info for use outside the lock
                                resumed_info = Some(info.clone());
                            }
                        }
                        
                        // Only send event if we actually resumed
                        if should_resume {
                            // Ensure the state is properly persisted
                            if let Some(info) = resumed_info {
                                *timer_info.lock().unwrap() = info;
                            }
                            
                            let send_result = event_tx.send(TimerEvent::Resumed).await;
                            if send_result.is_err() {
                                println!("Failed to send resume event");
                            }
                        }
                    }
                    
                    TimerCommand::Stop => {
                        // Update timer state
                        {
                            let mut info = timer_info.lock().unwrap();
                            info.state = TimerState::Idle;
                            info.current_phase = None;
                            info.time_remaining = None;
                            info.start_time = None;
                            info.pause_time = None;
                            
                            // Save state after stopping
                            save_timer_state(&info);
                        }
                        
                        // Send event after releasing the lock
                        let send_result = event_tx.send(TimerEvent::Stopped).await;
                        if send_result.is_err() {
                            println!("Failed to send stop event");
                        }
                    }
                    
                    TimerCommand::Skip => {
                        // Implement skip logic - clone data first to avoid borrow issues
                        let (workflow_opt, phase_opt, is_running_or_paused) = {
                            let info = timer_info.lock().unwrap();
                            (
                                info.current_workflow.clone(),
                                info.current_phase.clone(), 
                                info.state == TimerState::Running || info.state == TimerState::Paused
                            )
                        };
                        
                        if !is_running_or_paused {
                            continue;
                        }
                        
                        if let (Some(workflow), Some(current_phase)) = (workflow_opt, phase_opt) {
                            // Find the current phase index
                            if let Some(current_index) = workflow.phases.iter().position(|p| p.name == current_phase.name) {
                                // Move to the next phase
                                if current_index + 1 < workflow.phases.len() {
                                    let next_phase = workflow.phases[current_index + 1].clone();
                                    let was_paused;
                                    
                                    // Update timer info with the new phase
                                    {
                                        let mut info = timer_info.lock().unwrap();
                                        was_paused = info.state == TimerState::Paused;
                                        info.current_phase = Some(next_phase.clone());
                                        info.time_remaining = Some(Duration::minutes(next_phase.duration as i64));
                                        info.elapsed_time = Duration::zero();
                                        
                                        if was_paused {
                                            info.state = TimerState::Running;
                                            info.pause_time = None;
                                        }
                                        
                                        // Save state after skipping
                                        save_timer_state(&info);
                                    }
                                    
                                    // Send event after releasing the lock
                                    let send_result = event_tx.send(TimerEvent::PhaseChanged {
                                        phase: next_phase,
                                    }).await;
                                    if send_result.is_err() {
                                        println!("Failed to send phase changed event");
                                    }
                                } else {
                                    // End of workflow
                                    {
                                        let mut info = timer_info.lock().unwrap();
                                        info.state = TimerState::Completed;
                                        info.current_phase = None;
                                        info.time_remaining = None;
                                        
                                        // Save state after completion
                                        save_timer_state(&info);
                                    }
                                    
                                    // Send event after releasing the lock
                                    let send_result = event_tx.send(TimerEvent::Completed).await;
                                    if send_result.is_err() {
                                        println!("Failed to send completed event");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// A new task to consume events from the channel
async fn event_consumer_task(mut event_rx: mpsc::Receiver<TimerEvent>) {
    while let Some(event) = event_rx.recv().await {
        match event {
            TimerEvent::Started { .. } => {
                // Handle start event
            },
            TimerEvent::PhaseChanged { .. } => {
                // Handle phase change event
            },
            TimerEvent::Paused => {
                // Handle pause event
            },
            TimerEvent::Resumed => {
                // Handle resume event
            },
            TimerEvent::Stopped => {
                // Handle stop event
            },
            TimerEvent::Completed => {
                // Handle completion event
            },
        }
    }
}

// Helper function to save timer state to persistence
fn save_timer_state(info: &TimerInfo) {
    let persistent_state = persistence::PersistentState {
        timer_state: info.state.clone(),
        current_phase: info.current_phase.clone(),
        current_status: info.current_status.clone(),
        current_workflow: info.current_workflow.clone(),
        start_time: info.start_time,
        elapsed_seconds: info.elapsed_time.num_seconds() as u64,
        last_saved: Local::now(),
    };
    
    if let Err(e) = persistence::update(persistent_state) {
        eprintln!("Failed to save timer state: {}", e);
    }
} 