#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

PRODLOG_CMD_PREFIX=$'\x1A(dd0d3038-1d43-11f0-9761-022486cd4c38) PRODLOG:'
CMD_IS_INACTIVE="IS CURRENTLY INACTIVE"
CMD_ARE_YOU_RUNNING="PRODLOG, ARE YOU RUNNING?"
CMD_START_CAPTURE="START CAPTURE"
CMD_STOP_CAPTURE="STOP CAPTURE"
REPLY_YES_PRODLOG_IS_RUNNING="PRODLOG IS RUNNING"

# Function to send commands to prodlog via stdout
# Usage: send_command "COMMAND_NAME" "arg1" "arg2" ...
send_command() {
    local cmd="$1"
    shift
    local encoded_args=""
    local arg
    for arg in "$@"; do
        # Use base64 -w0 to prevent line wrapping
        encoded_args+=$(echo -n "$arg" | base64 -w0)
        encoded_args+=';'
    done
    printf "\n%s%s;%s\n" "$PRODLOG_CMD_PREFIX" "$cmd" "$encoded_args"
}

# Function to print help message
print_help() {
    echo "Usage: $0 run <command> [args...]"
    echo "Record a command and its output in prodlog. An instance of prodlog_server must be running."
    echo ""
    echo "Testing if prodlog_server is running:"
    send_command "$CMD_IS_INACTIVE"
}

# --- Main Script Logic ---

# Check if the first argument is 'run'
if [[ "$1" != "run" ]]; then
    print_help
    exit 1
fi

# Remove the 'run' argument
shift

# Check if a command was provided
if [[ $# -eq 0 ]]; then
    echo "Error: No command provided to run."
    print_help
    exit 1
fi

# Check if prodlog is running
send_command "$CMD_ARE_YOU_RUNNING"

# Read response from stdin with a 1-second timeout
if ! read -t 1 response; then
    echo "Error: Timeout waiting for prodlog response. Is it running?" >&2
    # Attempt to read any leftover partial input to clear buffer (optional)
    # read -t 0.1 -n 10000 discard || true 
    exit 1
fi

# Trim potential leading/trailing whitespace (though python readline().strip() is more robust)
response=$(echo "$response" | xargs) 

if [[ "$response" != "$REPLY_YES_PRODLOG_IS_RUNNING" ]]; then
    echo "Error: Unexpected response from prodlog: '$response'" >&2
    exit 1
fi

# Get metadata
hostname=$(hostname)
cwd=$(pwd)
cmd_str="$*" # Capture the command and arguments as a single string

# Send start marker
send_command "$CMD_START_CAPTURE" "$hostname" "$cwd" "$cmd_str"

# Execute the command, passing remaining arguments
# The trap ensures STOP_CAPTURE is sent even if the command fails
trap 'send_command "$CMD_STOP_CAPTURE"' EXIT

"$@"
exit_status=$?

# Note: The trap handles sending STOP_CAPTURE upon exit

exit $exit_status