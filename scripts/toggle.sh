#!/bin/bash

# Toggle script for tomato-clock
# This script helps to better handle the toggle action between start/pause

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/tomato-clock"
STATE_FILE="$CONFIG_DIR/state.json"
WAYBAR_OUTPUT_FILE="$CONFIG_DIR/waybar-output.json"
LOCK_FILE="/tmp/tomato-clock-toggle.lock"
TOMATO_CLOCK_BIN="$HOME/.local/bin/tomato-clock"
LOGFILE="/tmp/tomato-clock-toggle.log"

# Ensure only one instance is running
if [ -f "$LOCK_FILE" ]; then
    pid=$(cat "$LOCK_FILE")
    if ps -p "$pid" > /dev/null; then
        echo "$(date): Another toggle.sh is running (PID: $pid), exiting" >> "$LOGFILE"
        exit 0
    fi
fi

# Create lock file
echo $$ > "$LOCK_FILE"

# Log start of script with timestamp
echo "$(date): Toggle script started" >> "$LOGFILE"

# Check if the daemon is running, start if not
if ! ps aux | grep -q "[t]omato-clock daemon"; then
    echo "$(date): Daemon not running, starting daemon" >> "$LOGFILE"
    $TOMATO_CLOCK_BIN daemon &
    # Wait for daemon to initialize
    sleep 1
fi

# Reset timer completely first to ensure clean state
echo "$(date): Stopping timer to ensure clean state" >> "$LOGFILE"
$TOMATO_CLOCK_BIN stop
sleep 1

# Ensure we're starting from a clean state
echo "$(date): Starting new timer" >> "$LOGFILE"
$TOMATO_CLOCK_BIN start
sleep 1

# Final info to verify state
$TOMATO_CLOCK_BIN info > /tmp/tomato-clock-status.txt 2>&1
echo "$(date): Final timer info:" >> "$LOGFILE"
cat /tmp/tomato-clock-status.txt >> "$LOGFILE"

# Remove lock file
rm -f "$LOCK_FILE"

exit 0 