use clap::{Parser, Subcommand};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::signal::ctrl_c;
use std::time::Duration as StdDuration;

mod config;
mod persistence;
mod status;
mod timer;
mod waybar;
mod workflow;

use crate::status::StatusManager;
use crate::timer::{Timer, TimerCommand, TimerState};
use crate::waybar::update_waybar_output;
use crate::workflow::{Workflow, WorkflowManager};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the timer with the current or specified workflow
    Start {
        /// Specify the workflow to use
        #[arg(short, long)]
        workflow: Option<String>,
        
        /// Specify the status to use
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Stop the timer
    Stop,
    /// Pause the timer
    Pause,
    /// Resume the timer
    Resume,
    /// Skip the current phase
    Skip,
    /// Set the current status
    Status {
        /// The status to set (e.g., work, study, chilling)
        name: String,
    },
    /// Manage workflows
    Workflow {
        #[command(subcommand)]
        action: WorkflowCommands,
    },
    /// Run as a daemon for Waybar integration
    Daemon,
    /// Display the current timer information
    Info,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// List all available workflows
    List,
    /// Add a new workflow
    Add {
        /// Name of the workflow
        name: String,
        /// Phases in format "name:duration_mins,name:duration_mins,..."
        phases: String,
    },
    /// Remove a workflow
    Remove {
        /// Name of the workflow to remove
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap_or_else(|e| {
        eprintln!("Failed to initialize logger: {}", e);
    });

    let cli = Cli::parse();

    // Initialize configuration
    match config::init(cli.config.clone()) {
        Ok(_) => info!("Configuration loaded"),
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    }

    // Initialize persistence
    match persistence::init() {
        Ok(_) => info!("Persistence initialized"),
        Err(e) => {
            error!("Failed to initialize persistence: {}", e);
            std::process::exit(1);
        }
    }

    // Create managers
    let status_manager = StatusManager::new();
    let workflow_manager = WorkflowManager::new();
    
    // Create timer
    let timer = Arc::new(AsyncMutex::new(Timer::new().await));

    // Create global lock to ensure only one command runs at a time
    // Keeping this for future use, but marking as unused to suppress warnings
    let _command_lock = Arc::new(AsyncMutex::new(()));

    // Process commands
    match cli.command {
        Some(Commands::Start { workflow, status }) => {
            info!("Starting timer with workflow: {:?}, status: {:?}", workflow, status);
            
            let workflow_obj = if let Some(workflow_name) = workflow {
                workflow_manager.get_workflow(&workflow_name).ok_or_else(|| {
                    error!("Workflow '{}' not found", workflow_name);
                    "Workflow not found"
                })?
            } else {
                let default_workflow_name = config::get().default_workflow;
                workflow_manager.get_workflow(&default_workflow_name).ok_or_else(|| {
                    error!("Default workflow '{}' not found", default_workflow_name);
                    "Default workflow not found"
                })?
            };
            
            let status_obj = if let Some(status_name) = status {
                status_manager.get_status(&status_name).ok_or_else(|| {
                    error!("Status '{}' not found", status_name);
                    "Status not found"
                })?
            } else {
                let default_status_name = config::get().default_status;
                status_manager.get_status(&default_status_name).ok_or_else(|| {
                    error!("Default status '{}' not found", default_status_name);
                    "Default status not found"
                })?
            };
            
            let timer_lock = timer.lock().await;
            timer_lock.send_command(TimerCommand::Start {
                workflow: Some(workflow_obj.clone()),
                status: Some(status_obj.clone()),
            }).await?;
            
            // Update waybar
            update_waybar_output(&timer_lock.get_info())?;
            
            info!("Timer started with workflow '{}' and status '{}'", 
                  workflow_obj.name, status_obj.name);
        }
        Some(Commands::Stop) => {
            info!("Stopping timer");
            
            let timer_lock = timer.lock().await;
            timer_lock.send_command(TimerCommand::Stop).await?;
            
            // Update waybar
            update_waybar_output(&timer_lock.get_info())?;
            
            info!("Timer stopped");
        }
        Some(Commands::Pause) => {
            info!("Pausing timer");
            
            let timer_lock = timer.lock().await;
            
            // Check if timer is already paused
            let info = timer_lock.get_info();
            if info.state == TimerState::Paused {
                info!("Timer is already paused");
                return Ok(());
            }
            
            // Send pause command
            timer_lock.send_command(TimerCommand::Pause).await?;
            
            // Get updated info and update waybar
            let updated_info = timer_lock.get_info();
            update_waybar_output(&updated_info)?;
            
            info!("Timer paused");
        }
        Some(Commands::Resume) => {
            info!("Resuming timer");
            
            let timer_lock = timer.lock().await;
            timer_lock.send_command(TimerCommand::Resume).await?;
            
            // Update waybar
            update_waybar_output(&timer_lock.get_info())?;
            
            info!("Timer resumed");
        }
        Some(Commands::Skip) => {
            info!("Skipping current phase");
            
            let timer_lock = timer.lock().await;
            timer_lock.send_command(TimerCommand::Skip).await?;
            
            // Update waybar
            update_waybar_output(&timer_lock.get_info())?;
            
            info!("Phase skipped");
        }
        Some(Commands::Status { name }) => {
            info!("Setting status to: {}", name);
            
            // Get the status from the manager
            if let Some(status) = status_manager.get_status(&name) {
                // Start the timer with current workflow but new status
                let timer_lock = timer.lock().await;
                let info = timer_lock.get_info();
                
                timer_lock.send_command(TimerCommand::Start {
                    workflow: info.current_workflow,
                    status: Some(status.clone()),
                }).await?;
                
                // Update waybar
                update_waybar_output(&timer_lock.get_info())?;
                
                info!("Status changed to '{}'", name);
            } else {
                error!("Status '{}' not found", name);
                return Err("Status not found".into());
            }
        }
        Some(Commands::Workflow { action }) => match action {
            WorkflowCommands::List => {
                info!("Listing workflows");
                
                let workflows = workflow_manager.list_workflows();
                println!("Available workflows:");
                
                for workflow in workflows {
                    println!("- {} ({})", 
                        workflow.name, 
                        workflow.description.unwrap_or_else(|| "No description".to_string()));
                    
                    println!("  Phases:");
                    for phase in workflow.phases {
                        println!("  - {} ({} minutes)", phase.name, phase.duration);
                    }
                    println!();
                }
            }
            WorkflowCommands::Add { name, phases } => {
                info!("Adding workflow '{}' with phases: {}", name, phases);
                
                // Parse phases
                match Workflow::parse_phases(&phases) {
                    Ok(parsed_phases) => {
                        let workflow = Workflow::new(&name)
                            .with_phases(parsed_phases)
                            .with_repeatable(true);
                        
                        match workflow_manager.add_workflow(workflow) {
                            Ok(_) => info!("Workflow '{}' added successfully", name),
                            Err(e) => {
                                error!("Failed to add workflow: {}", e);
                                return Err(e.into());
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse phases: {}", e);
                        return Err(e.into());
                    }
                }
            }
            WorkflowCommands::Remove { name } => {
                info!("Removing workflow: {}", name);
                
                match workflow_manager.remove_workflow(&name) {
                    Ok(_) => info!("Workflow '{}' removed successfully", name),
                    Err(e) => {
                        error!("Failed to remove workflow: {}", e);
                        return Err(e.into());
                    }
                }
            }
        },
        Some(Commands::Info) => {
            let timer_lock = timer.lock().await;
            let info = timer_lock.get_info();
            
            println!("Timer State: {:?}", info.state);
            
            if let Some(workflow) = &info.current_workflow {
                println!("Current Workflow: {}", workflow.name);
            } else {
                println!("Current Workflow: None");
            }
            
            if let Some(status) = &info.current_status {
                println!("Current Status: {}", status.name);
            } else {
                println!("Current Status: None");
            }
            
            if let Some(phase) = &info.current_phase {
                println!("Current Phase: {} ({} minutes)", phase.name, phase.duration);
            } else {
                println!("Current Phase: None");
            }
            
            if let Some(remaining) = &info.time_remaining {
                let total_seconds = remaining.num_seconds();
                let minutes = total_seconds / 60;
                let seconds = total_seconds % 60;
                println!("Time Remaining: {:02}:{:02}", minutes, seconds);
            } else {
                println!("Time Remaining: None");
            }
            
            let elapsed_seconds = info.elapsed_time.num_seconds();
            let elapsed_minutes = elapsed_seconds / 60;
            let elapsed_secs = elapsed_seconds % 60;
            println!("Elapsed Time: {:02}:{:02}", elapsed_minutes, elapsed_secs);
        }
        Some(Commands::Daemon) => {
            info!("Starting in daemon mode");
            
            // Create a timer to update waybar periodically
            let timer_clone = Arc::clone(&timer);
            
            // Create a task to handle signals for clean shutdown
            tokio::spawn(async move {
                match ctrl_c().await {
                    Ok(()) => {
                        info!("Received shutdown signal, saving state and exiting");
                        
                        // Last state update before shutdown
                        let timer_lock = timer_clone.lock().await;
                        let info = timer_lock.get_info();
                        update_waybar_output(&info).unwrap_or_else(|e| {
                            error!("Failed to update waybar output: {}", e);
                        });
                        
                        std::process::exit(0);
                    },
                    Err(e) => error!("Failed to listen for shutdown signal: {}", e),
                }
            });
            
            // Set up timer state socket listener for IPC
            // TODO: Implement IPC socket if needed
            
            // Start the main daemon loop
            let timer_clone = Arc::clone(&timer);
            loop {
                // Get timer info and update waybar
                let timer_lock = timer_clone.lock().await;
                let info = timer_lock.get_info();
                update_waybar_output(&info)?;
                
                // Sleep for a short duration
                drop(timer_lock); // Release the lock before sleeping
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
        None => {
            info!("No command specified, starting in interactive mode");
            // Try to connect to running daemon or start a simple CLI
            println!("No command specified. Use --help to see available commands.");
        }
    }

    Ok(())
}

// Helper function to execute commands with a shared lock
// Keeping this for future use when we need to enforce command serialization
#[allow(dead_code)]
async fn execute_command_with_lock(
    timer: &Arc<AsyncMutex<Timer>>,
    command_lock: &Arc<AsyncMutex<()>>,
    operation: impl FnOnce(&Timer) -> Result<(), Box<dyn std::error::Error>> + Send,
) -> Result<(), Box<dyn std::error::Error>> {
    // Acquire command lock to prevent concurrent commands
    let _guard = command_lock.lock().await;
    
    // Acquire timer lock
    let timer_lock = timer.lock().await;
    
    // Execute the operation
    operation(&timer_lock)?;
    
    // Update waybar
    update_waybar_output(&timer_lock.get_info())?;
    
    // Add a small delay to ensure persistence has time to complete
    tokio::time::sleep(StdDuration::from_millis(100)).await;
    
    Ok(())
}
