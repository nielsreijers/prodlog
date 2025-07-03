# Prodlog

A terminal logging tool that captures command execution and file editing sessions on remote servers with a beautiful web interface for viewing logs.

## Overview

Prodlog consists of two components that work together:

1. **`prodlog_record`** - A local terminal recorder that runs on your machine
2. **`prodlog`** - A lightweight bash script that runs on remote servers to mark commands for capture

The magic happens through special control sequences: when you run `prodlog run <command>` or `prodlog edit <file>` on a remote server, it prints special markers to stdout that are detected by the local `prodlog_record` process. All terminal output between `START CAPTURE` and `STOP CAPTURE` markers gets logged along with metadata like hostname, working directory, exit codes, and execution time.

## How It Works

1. Start `prodlog_record` on your local machine - it spawns a terminal session and starts a web UI
2. SSH to your remote server (or run commands locally)
3. Copy the `prodlog` script to the remote server
4. Use `prodlog run <command>` or `prodlog edit <file>` to mark commands for logging
5. View captured logs in the web interface at `http://localhost:5000`

The system captures:
- **Command runs**: Full terminal output, exit codes, execution time
- **File edits**: Before/after file contents with diff visualization
- **Metadata**: Hostname, working directory, user, timestamps

## Installation

### Building from Source

Make sure [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and [Node.js](https://nodejs.org/) are installed.

```bash
# Clone the repository
git clone https://github.com/nielsreijers/prodlog.git
cd prodlog

# Build the React UI
./build-react.sh

# Install the binary
cargo install --path .
```

The React UI is embedded into the binary at compile time, so you only need to build it once. After installation, `cargo install` will work without requiring Node.js on the target system.

### Remote Script Setup

Install the `prodlog` bash script to your remote servers and make it executable:

```bash
export HTTPS_PROXY=...
mkdir -p ~/.local/bin
curl -s https://raw.githubusercontent.com/nielsreijers/prodlog/rust/prodlog > ~/.local/bin/prodlog
chmod +x ~/.local/bin/prodlog
```

## Usage

### Local Recording (`prodlog_record`)

Start the local recorder on your machine:

```bash
prodlog_record [OPTIONS]
```

#### Options

```
--dir <DIR>                  Directory to store production logs (default: ~/.local/share/prodlog)
--port <PORT>                Port for the web UI (default: 5000)
--import <FILE>              Import an existing prodlog JSON or SQLite file.
--cmd <CMD>                  Initial command to run (default: /bin/bash). This can be used to
                             create macros that start prodlog and immediately run ssh to connect
                             to a remote server
--ui-background <HEX_COLOUR> Background color for the web UI (default: #FFFFFF)
```

#### Examples

```bash
# Basic usage - starts bash locally
prodlog_record

# Start with SSH to remote server
prodlog_record --cmd "ssh user@server.example.com"

# Use custom port and data directory
prodlog_record --port 8080 --dir ~/my-logs

# Import existing logs
prodlog_record --import old-logs.json

# Custom UI background color
prodlog_record --ui-background "#1e1e1e"
```

Once started, `prodlog_record` will:
- Open a terminal session (bash by default, or your specified command)
- Start a web UI at `http://localhost:5000` (or your specified port)
- Log all marked commands to JSON, SQLite, and Obsidian formats
  The sinks are a work in progress and JSON and Obsidian are old
  experiments that will likely be removed in a future version, but a
  MySQL sink will be added at some point.

### Remote Command Capture (`prodlog`)

On remote servers, use the `prodlog` script to mark commands for capture:

```bash
prodlog run [-m <message>] [-s] <command> [args...]
prodlog edit [-m <message>] [-s] <filename>
```

#### Options

- `-m <message>` - Optional message to log with the command or edit
- `-s` - Use sudo to run the command or edit the file

#### Examples

```bash
# Capture a simple command
prodlog run ls -la

# Capture with a descriptive message
prodlog run -m "Checking disk usage" df -h

# Capture a command with sudo
prodlog run -s systemctl restart nginx

# Edit a file and capture changes
prodlog edit /etc/nginx/nginx.conf

# Edit with sudo and add a message
prodlog edit -s -m "Updating SSL configuration" /etc/nginx/sites-available/default
```

### Web Interface

The web UI provides:

- **Filterable log entries** by date, hostname, command, or message
- **Command output viewer** with full terminal output and xterm.js rendering
- **Diff viewer** for file edits with syntax highlighting
- **Entry editing** to add/modify messages and metadata
- **Search functionality** across commands and messages
- **Export capabilities** to JSON format

Access the web interface at `http://localhost:5000` (or your configured port).

## Data Storage

Prodlog stores data in multiple formats:

- **SQLite database** (`prodlog.sqlite`) - Primary storage for the web UI
- **JSON file** (`prodlog.json`) - Human-readable backup format
- **Obsidian notes** - Individual markdown files for each entry

All files are stored in the configured data directory (default: `~/.local/share/prodlog`).

## License

This software is provided under a coffee license. Continued use requires payment, though enforcement is relaxed and based on the honor system. See the `LICENSE` file for full terms.
