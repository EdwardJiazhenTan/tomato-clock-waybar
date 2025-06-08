# Desktop Notification System for Tomato Clock

## Overview

This document outlines the plan for implementing desktop notifications in the Tomato Clock application for Linux systems using Hyprland, Wayland, and Arch Linux.

## Requirements

- Notify users when timer phases change
- Notify users when a timer is completed
- Allow users to customize notification settings
- Integrate seamlessly with modern Linux desktop environments

## Technology Stack

- **Notification Library**: `notify-rust` (already a dependency)
- **Desktop Environment**: Hyprland/Wayland
- **Target OS**: Arch Linux
- **Notification Server**: Mako (recommended for Hyprland/Wayland)

## Implementation Steps

### 1. Notification Service Module

Create a new `notification.rs` module with the following features:

```rust
pub struct NotificationService {
    enabled: bool,
    timeout_ms: u32,
    sound_enabled: bool,
    sound_file: Option<String>,
}

impl NotificationService {
    pub fn new(config: &Config) -> Self {
        Self {
            enabled: config.notification_enabled,
            timeout_ms: config.notification_timeout_ms.unwrap_or(5000),
            sound_enabled: config.notification_sound_enabled.unwrap_or(false),
            sound_file: config.notification_sound_file.clone(),
        }
    }

    pub fn send_phase_change_notification(&self, phase: &Phase, status: &Status) -> Result<(), String> {
        // Implementation
    }

    pub fn send_timer_completed_notification(&self, workflow: &Workflow) -> Result<(), String> {
        // Implementation
    }

    pub fn send_generic_notification(&self, summary: &str, body: &str, urgency: NotificationUrgency) -> Result<(), String> {
        // Implementation
    }
}
```

### 2. Integration with Event System

Modify the `event_consumer_task` in `timer.rs` to handle notification events:

```rust
async fn event_consumer_task(mut event_rx: mpsc::Receiver<TimerEvent>, notification_service: Arc<Mutex<NotificationService>>) {
    while let Some(event) = event_rx.recv().await {
        match event {
            TimerEvent::PhaseChanged { phase } => {
                // Get the current timer info
                let timer_info = /* ... */;
                if let Some(status) = &timer_info.current_status {
                    notification_service.lock().unwrap()
                        .send_phase_change_notification(&phase, status)
                        .unwrap_or_else(|e| eprintln!("Failed to send notification: {}", e));
                }
            },
            TimerEvent::Completed => {
                // Get the current timer info
                let timer_info = /* ... */;
                if let Some(workflow) = &timer_info.current_workflow {
                    notification_service.lock().unwrap()
                        .send_timer_completed_notification(workflow)
                        .unwrap_or_else(|e| eprintln!("Failed to send notification: {}", e));
                }
            },
            // Handle other events
            _ => {}
        }
    }
}
```

### 3. Configuration Extensions

Extend the `config.toml` schema to include notification settings:

```toml
# Enable or disable desktop notifications
notification_enabled = true

# Notification timeout in milliseconds
notification_timeout_ms = 5000

# Enable or disable notification sounds
notification_sound_enabled = false

# Path to custom notification sound file (optional)
# notification_sound_file = "/path/to/sound.wav"
```

### 4. Notification Implementation

Implement the notification functions using `notify-rust`:

```rust
pub fn send_phase_change_notification(&self, phase: &Phase, status: &Status) -> Result<(), String> {
    if !self.enabled {
        return Ok(());
    }

    let summary = format!("Phase Changed: {}", phase.name);
    let body = format!("Status: {}\nDuration: {} minutes", status.name, phase.duration);

    Notification::new()
        .summary(&summary)
        .body(&body)
        .icon("tomato-clock")
        .timeout(self.timeout_ms)
        .show()
        .map_err(|e| format!("Failed to send notification: {}", e))?;

    // Play sound if enabled
    if self.sound_enabled {
        // Implementation for playing sound
    }

    Ok(())
}
```

### 5. Hyprland/Wayland Integration

For proper integration with Hyprland/Wayland:

1. Ensure Mako notification daemon is installed and running:

   ```bash
   # Install Mako on Arch Linux
   sudo pacman -S mako

   # Start Mako in the user's startup script
   mako &
   ```

2. Configure Mako for optimal display:

   ```
   # ~/.config/mako/config
   max-visible=5
   default-timeout=5000
   icon-path=/usr/share/icons/Papirus-Dark

   # Style for tomato-clock notifications
   [app-name=tomato-clock]
   background-color=#ff5555
   text-color=#ffffff
   border-color=#ff79c6
   ```

3. Add an icon to the system:

   ```bash
   # Create icon directory if it doesn't exist
   mkdir -p ~/.local/share/icons/hicolor/scalable/apps/

   # Copy icon to the appropriate location
   cp /path/to/tomato-clock-icon.svg ~/.local/share/icons/hicolor/scalable/apps/tomato-clock.svg
   ```

### 6. Sound Support (Optional)

For systems that support it, implement sound notifications:

```rust
fn play_notification_sound(&self) -> Result<(), String> {
    if !self.sound_enabled {
        return Ok(());
    }

    let sound_file = self.sound_file.as_ref().unwrap_or(&"default".to_string());

    // Use rodio or similar crate to play sounds
    // Alternatively, use the system's sound server via dbus

    Ok(())
}
```

## Testing

Test the notification system with:

1. Different desktop environments (Hyprland, Sway, other Wayland compositors)
2. Different notification servers (Mako, Dunst)
3. Different notification settings (with/without sound, different timeouts)
4. Different scenarios (phase change, timer completion)

## Future Enhancements

1. **Action Buttons**: Add buttons to notifications for quick actions (e.g., "Skip", "Pause")
2. **Notification Queue**: Manage notification queue to prevent overwhelming the user
3. **Do Not Disturb Mode**: Respect system DND settings
4. **Custom Icons**: Allow users to set custom icons for different phases
5. **Urgency Levels**: Set different urgency levels for different events

## Resources

- notify-rust documentation: https://docs.rs/notify-rust/latest/notify_rust/
- Mako documentation: https://github.com/emersion/mako
- Hyprland documentation: https://wiki.hyprland.org/
- Arch Linux notification setup: https://wiki.archlinux.org/title/Desktop_notifications
