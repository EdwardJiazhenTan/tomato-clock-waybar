#!/bin/bash

# Simple toggle script for tomato-clock
# This script provides a simple pause/resume toggle for the tomato clock

TOMATO_CLOCK_BIN="$HOME/.local/bin/tomato-clock"
LOGFILE="/tmp/tomato-clock-simple-toggle.log"

# Log start
echo "$(date): Simple toggle script started" >> "$LOGFILE"

# Get current status
STATUS=$($TOMATO_CLOCK_BIN info | grep "Timer State" | awk '{print $3}')
echo "$(date): Current status: $STATUS" >> "$LOGFILE"

# If Running, pause; if anything else, resume or start
if [ "$STATUS" = "Running" ]; then
    echo "$(date): Timer is running, pausing..." >> "$LOGFILE"
    $TOMATO_CLOCK_BIN pause
else 
    if [ "$STATUS" = "Paused" ]; then
        echo "$(date): Timer is paused, resuming..." >> "$LOGFILE"
        $TOMATO_CLOCK_BIN resume
    else
        echo "$(date): Timer is not running, starting..." >> "$LOGFILE"
        $TOMATO_CLOCK_BIN start
    fi
fi

# Log final status
echo "$(date): Action completed" >> "$LOGFILE"
echo "-----------------------------------" >> "$LOGFILE"

exit 0 