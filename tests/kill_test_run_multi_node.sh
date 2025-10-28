#!/bin/bash

# Define the list of commands to search for
declare -a commands=(
    "linera --with-wallet 1 service --port 8081"
    "linera --with-wallet 2 service --port 8082"
    "linera --with-wallet 3 service --port 8083"
)

# Loop through each command to find and terminate its process
for cmd in "${commands[@]}"; do
    # Use pgrep to find the PID(s) matching the full command line
    pid=$(pgrep -f "$cmd")

    if [ -n "$pid" ]; then
        echo "Terminating process with PID(s): $pid for command: $cmd"
        kill "$pid"
    else
        echo "No process found for command: $cmd"
    fi
done