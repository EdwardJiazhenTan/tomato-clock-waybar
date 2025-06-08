use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Status {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            name: "work".to_string(),
            description: Some("Working on tasks".to_string()),
            color: Some("#ff5555".to_string()),
            icon: Some("🔨".to_string()),
        }
    }
}

impl Status {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
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

#[derive(Debug)]
pub struct StatusManager {
    statuses: Arc<Mutex<HashMap<String, Status>>>,
}

impl StatusManager {
    pub fn new() -> Self {
        let mut statuses = HashMap::new();
        
        // Add default statuses
        statuses.insert(
            "work".to_string(),
            Status::new("work")
                .with_description("Working on tasks")
                .with_color("#ff5555")
                .with_icon("🔨"),
        );
        
        statuses.insert(
            "study".to_string(),
            Status::new("study")
                .with_description("Studying or learning")
                .with_color("#f1fa8c")
                .with_icon("📚"),
        );
        
        statuses.insert(
            "chilling".to_string(),
            Status::new("chilling")
                .with_description("Taking a break")
                .with_color("#8be9fd")
                .with_icon("☕"),
        );
        
        Self {
            statuses: Arc::new(Mutex::new(statuses)),
        }
    }
    
    #[allow(dead_code)]
    pub fn add_status(&self, status: Status) -> Result<(), &'static str> {
        let mut statuses = self.statuses.lock().unwrap();
        if statuses.contains_key(&status.name) {
            return Err("Status with this name already exists");
        }
        
        statuses.insert(status.name.clone(), status);
        Ok(())
    }
    
    pub fn get_status(&self, name: &str) -> Option<Status> {
        let statuses = self.statuses.lock().unwrap();
        statuses.get(name).cloned()
    }
    
    #[allow(dead_code)]
    pub fn remove_status(&self, name: &str) -> Result<(), &'static str> {
        let mut statuses = self.statuses.lock().unwrap();
        if !statuses.contains_key(name) {
            return Err("Status with this name does not exist");
        }
        
        statuses.remove(name);
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn list_statuses(&self) -> Vec<Status> {
        let statuses = self.statuses.lock().unwrap();
        statuses.values().cloned().collect()
    }
    
    #[allow(dead_code)]
    pub fn update_status(&self, status: Status) -> Result<(), &'static str> {
        let mut statuses = self.statuses.lock().unwrap();
        if !statuses.contains_key(&status.name) {
            return Err("Status with this name does not exist");
        }
        
        statuses.insert(status.name.clone(), status);
        Ok(())
    }
} 