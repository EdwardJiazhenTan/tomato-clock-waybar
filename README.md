# Tomato Clock for Waybar

A simple Pomodoro timer designed to integrate with [Waybar](https://github.com/Alexays/Waybar) on Linux systems.

## Features

- Classic Pomodoro technique timer
- Seamless Waybar integration
- Configurable work and break durations
- Persistent state across restarts
- Multiple notification methods
- Socket-based communication (improved reliability)

## Installation

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Waybar

### Building from source

1. Clone the repository:

   ```
   git clone https://github.com/yourusername/tomato-clock-waybar.git
   cd tomato-clock-waybar
   ```

2. Build the project:

   ```
   cargo build --release
   ```

3. Install the binary:

   ```
   cp target/release/tomato-clock ~/.local/bin/
   ```

## Usage

### Basic Commands

```bash
# Start the timer
tomato-clock start

# Stop the timer
tomato-clock stop

# Pause the timer
tomato-clock pause

# Resume the timer
tomato-clock resume

# Skip the current phase
tomato-clock skip

# Show timer information
tomato-clock info

# Run the daemon (required for Waybar integration)
tomato-clock daemon
```

### Integration with Waybar

There are two methods to integrate with Waybar:

#### Method 1: Direct Integration (Simpler)

Add this to your Waybar config:

```json
"custom/tomato": {
    "exec": "cat ~/.config/tomato-clock/waybar-output.json",
    "return-type": "json",
    "interval": 1,
    "on-click": "~/.local/bin/tomato-clock start",
    "on-click-middle": "~/.local/bin/tomato-clock stop",
    "on-click-right": "~/.local/bin/tomato-clock skip"
}
```

#### Method 2: Socket-based Integration (Recommended)

For improved reliability and to fix "Failed to send xxx event" errors, use the socket-based integration:

1. Install the socket server:

   ```
   cd sockets
   chmod +x install_socket_server.sh
   ./install_socket_server.sh
   ```

2. This will automatically:
   - Install the required scripts
   - Configure Waybar
   - Create a systemd service to auto-start the socket server
   - Apply the necessary styles

See `sockets/README.md` for more details on the socket-based integration.

### Styling in Waybar

Add these styles to your Waybar CSS:

```css
#custom-tomato {
  padding: 0 0.6em;
  margin: 0 5px;
  border-radius: 5px;
  background-color: #2d3436;
  color: #e84393;
}

#custom-tomato.running {
  background-color: #55efc4;
  color: #2d3436;
  font-weight: bold;
}

#custom-tomato.paused {
  background-color: #ffeaa7;
  color: #2d3436;
}

#custom-tomato.completed {
  background-color: #74b9ff;
  color: #2d3436;
}
```

## Configuration

Create a configuration file at `~/.config/tomato-clock/config.toml`:

```toml
[timer]
work_duration = 25    # Work duration in minutes
break_duration = 5    # Break duration in minutes
long_break_duration = 15    # Long break duration in minutes
long_break_interval = 4    # Number of work sessions before a long break

[notification]
sound = true    # Enable sound notifications
desktop = true  # Enable desktop notifications
```

## Troubleshooting

If you encounter issues with Waybar integration:

1. Ensure the daemon is running:

   ```
   tomato-clock daemon
   ```

2. Check if the output file exists:

   ```
   cat ~/.config/tomato-clock/waybar-output.json
   ```

3. If experiencing "Failed to send xxx event" errors, use the socket-based integration method described above.

## Project Structure

- `src/` - Rust source code
- `config/` - Configuration examples
- `scripts/` - Helper scripts for Waybar integration
- `sockets/` - Socket-based integration files

## Development Roadmap

See [project_plan.md](project_plan.md) for the development roadmap.

## Notification System

See [notification_plan.md](notification_plan.md) for details on the notification system.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
