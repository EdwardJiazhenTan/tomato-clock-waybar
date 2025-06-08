#!/bin/bash

# Tomato Clock Waybar Module Script
# This script reads the output file from the tomato-clock app and displays it in Waybar

# Configuration
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/tomato-clock"
OUTPUT_FILE="$CONFIG_DIR/waybar-output.json"
# Use absolute path for tomato-clock
TOMATO_CLOCK_BIN="$HOME/.local/bin/tomato-clock"
TOGGLE_SCRIPT="$(dirname "$0")/toggle.sh"

# Add debug logging
LOGFILE="/tmp/tomato-clock-waybar.log"
echo "$(date): waybar-module.sh started with args: $@" >> $LOGFILE

# Check if the toggle script exists and is executable
if [ ! -x "$TOGGLE_SCRIPT" ]; then
    # Make it executable if it exists but isn't executable
    if [ -f "$TOGGLE_SCRIPT" ]; then
        chmod +x "$TOGGLE_SCRIPT"
        echo "$(date): Made toggle script executable" >> $LOGFILE
    else
        echo "Toggle script not found at $TOGGLE_SCRIPT" >&2
        echo "$(date): Toggle script not found at $TOGGLE_SCRIPT" >> $LOGFILE
    fi
fi

# Check if the output file exists
if [ ! -f "$OUTPUT_FILE" ]; then
    # Create a default output if the file doesn't exist
    mkdir -p "$CONFIG_DIR"
    echo '{"text":"🍅","tooltip":"Tomato Clock is not running","class":"idle"}' > "$OUTPUT_FILE"
    echo "$(date): Created default output file" >> $LOGFILE
fi

# Function to handle click events from Waybar
handle_click() {
    echo "$(date): Handling click event: $1" >> $LOGFILE
    case "$1" in
        1) # Left click - Start/Pause
            if [ -x "$TOGGLE_SCRIPT" ]; then
                echo "$(date): Executing toggle script" >> $LOGFILE
                "$TOGGLE_SCRIPT"
            else
                # Fallback to direct command if toggle script is not available
                echo "$(date): Toggle script not executable, using fallback" >> $LOGFILE
                "$TOMATO_CLOCK_BIN" toggle
            fi
            ;;
        2) # Middle click - Stop
            echo "$(date): Executing stop command" >> $LOGFILE
            "$TOMATO_CLOCK_BIN" stop
            ;;
        3) # Right click - Skip
            echo "$(date): Executing skip command" >> $LOGFILE
            "$TOMATO_CLOCK_BIN" skip
            ;;
    esac
}

# Check for click events from Waybar
if [ -n "$1" ]; then
    handle_click "$1"
    exit 0
elif [ -n "$WAYBAR_CLICK" ]; then
    handle_click "$WAYBAR_CLICK"
fi

# Output the content of the file
cat "$OUTPUT_FILE" 