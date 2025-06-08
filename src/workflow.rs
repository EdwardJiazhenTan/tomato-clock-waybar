use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use crate::config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub duration: u32, // Duration in minutes
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

impl Phase {
    pub fn new(name: &str, duration: u32) -> Self {
        Self {
            name: name.to_string(),
            duration,
            description: None,
            color: None,
            icon: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub phases: Vec<Phase>,
    pub description: Option<String>,
    pub repeatable: bool,
}

impl Default for Workflow {
    fn default() -> Self {
        Self {
            name: "Default Pomodoro".to_string(),
            phases: vec![
                Phase::new("Work", 25)
                    .with_description("Focus on work")
                    .with_color("#ff5555")
                    .with_icon("🔨"),
                Phase::new("Break", 5)
                    .with_description("Take a short break")
                    .with_color("#50fa7b")
                    .with_icon("☕"),
            ],
            description: Some("Standard Pomodoro technique workflow".to_string()),
            repeatable: true,
        }
    }
}

impl Workflow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            phases: Vec::new(),
            description: None,
            repeatable: true,
        }
    }

    pub fn with_phases(mut self, phases: Vec<Phase>) -> Self {
        self.phases = phases;
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_repeatable(mut self, repeatable: bool) -> Self {
        self.repeatable = repeatable;
        self
    }

    #[allow(dead_code)]
    pub fn add_phase(&mut self, phase: Phase) {
        self.phases.push(phase);
    }

    pub fn parse_phases(phases_str: &str) -> Result<Vec<Phase>, &'static str> {
        let parts = phases_str.split(',');
        let mut phases = Vec::new();

        for part in parts {
            let phase_parts: Vec<&str> = part.trim().split(':').collect();
            if phase_parts.len() != 2 {
                return Err("Invalid phase format, use 'name:duration'");
            }

            let name = phase_parts[0].trim();
            let duration = match phase_parts[1].trim().parse::<u32>() {
                Ok(duration) => duration,
                Err(_) => return Err("Invalid duration, must be a positive integer"),
            };

            phases.push(Phase::new(name, duration));
        }

        if phases.is_empty() {
            return Err("No phases provided");
        }

        Ok(phases)
    }
}

#[derive(Debug)]
pub struct WorkflowManager {
    workflows: Arc<Mutex<HashMap<String, Workflow>>>,
    workflow_file: PathBuf,
}

impl WorkflowManager {
    pub fn new() -> Self {
        let mut workflow_file = config::get_config_dir();
        workflow_file.push("workflows.json");
        
        let workflows = Self::load_workflows(&workflow_file).unwrap_or_else(|_| {
            let mut default_workflows = HashMap::new();
            
            // Add default workflows
            default_workflows.insert(
                "Default Pomodoro".to_string(),
                Workflow::default(),
            );
            
            default_workflows.insert(
                "Long Work Session".to_string(),
                Workflow::new("Long Work Session")
                    .with_phases(vec![
                        Phase::new("Work", 50)
                            .with_description("Focus on work")
                            .with_color("#ff5555")
                            .with_icon("🔨"),
                        Phase::new("Break", 10)
                            .with_description("Take a break")
                            .with_color("#50fa7b")
                            .with_icon("☕"),
                    ])
                    .with_description("Longer work sessions with longer breaks")
                    .with_repeatable(true),
            );
            
            default_workflows
        });
        
        Self {
            workflows: Arc::new(Mutex::new(workflows)),
            workflow_file,
        }
    }
    
    fn load_workflows(file_path: &PathBuf) -> Result<HashMap<String, Workflow>, String> {
        if !file_path.exists() {
            return Err("Workflow file does not exist".to_string());
        }
        
        let file_content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read workflow file: {}", e))?;
        
        serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse workflow file: {}", e))
    }
    
    fn save_workflows(&self) -> Result<(), String> {
        let workflows = self.workflows.lock().unwrap();
        
        // Create directory if it doesn't exist
        if let Some(parent) = self.workflow_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create workflows directory: {}", e))?;
            }
        }
        
        let json = serde_json::to_string_pretty(&*workflows)
            .map_err(|e| format!("Failed to serialize workflows: {}", e))?;
        
        fs::write(&self.workflow_file, json)
            .map_err(|e| format!("Failed to save workflows: {}", e))
    }
    
    pub fn add_workflow(&self, workflow: Workflow) -> Result<(), &'static str> {
        let mut workflows = self.workflows.lock().unwrap();
        if workflows.contains_key(&workflow.name) {
            return Err("Workflow with this name already exists");
        }
        
        workflows.insert(workflow.name.clone(), workflow);
        drop(workflows); // Release the lock before saving
        
        // Save changes to file
        if let Err(e) = self.save_workflows() {
            eprintln!("Failed to save workflows: {}", e);
        }
        
        Ok(())
    }
    
    pub fn get_workflow(&self, name: &str) -> Option<Workflow> {
        let workflows = self.workflows.lock().unwrap();
        workflows.get(name).cloned()
    }
    
    pub fn remove_workflow(&self, name: &str) -> Result<(), &'static str> {
        let mut workflows = self.workflows.lock().unwrap();
        if !workflows.contains_key(name) {
            return Err("Workflow with this name does not exist");
        }
        
        workflows.remove(name);
        drop(workflows); // Release the lock before saving
        
        // Save changes to file
        if let Err(e) = self.save_workflows() {
            eprintln!("Failed to save workflows: {}", e);
        }
        
        Ok(())
    }
    
    pub fn list_workflows(&self) -> Vec<Workflow> {
        let workflows = self.workflows.lock().unwrap();
        workflows.values().cloned().collect()
    }
    
    #[allow(dead_code)]
    pub fn update_workflow(&self, workflow: Workflow) -> Result<(), &'static str> {
        let mut workflows = self.workflows.lock().unwrap();
        if !workflows.contains_key(&workflow.name) {
            return Err("Workflow with this name does not exist");
        }
        
        workflows.insert(workflow.name.clone(), workflow);
        drop(workflows); // Release the lock before saving
        
        // Save changes to file
        if let Err(e) = self.save_workflows() {
            eprintln!("Failed to save workflows: {}", e);
        }
        
        Ok(())
    }
} 