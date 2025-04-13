# Prodlog

> **Note**: This entire project, including this README, was generated through conversations with an AI (Claude). The human user provided requirements and feedback, but did not write any code or documentation directly. The AI wrote every byte of code, documentation, and comments.

Prodlog is a command logging tool that records your terminal sessions, including commands, output, timing, and host information. It includes a web interface for browsing and searching through your recorded sessions.

## Quick Install

```bash
mkdir -p ~/.prodlog && \
curl -s https://raw.githubusercontent.com/nielsreijers/prodlog/main/prodlog > ~/.prodlog/prodlog && \
curl -s https://raw.githubusercontent.com/nielsreijers/prodlog/main/prodlog_ui > ~/.prodlog/prodlog_ui && \
chmod +x ~/.prodlog/prodlog ~/.prodlog/prodlog_ui && \
ln -sf ~/.prodlog/prodlog ~/.local/bin/prodlog && \
ln -sf ~/.prodlog/prodlog_ui ~/.local/bin/prodlog_ui
```

Note: 
- Make sure `~/.local/bin` is in your PATH. If not, add `export PATH="$HOME/.local/bin:$PATH"` to your `~/.bashrc` or `~/.zshrc`
- No sudo required - installs only for current user
- Requires Python 3 and Flask (`pip install flask`)

## Usage

```
usage: prodlog [-h] [--dir DIR] {run,record} ...

Record shell output

positional arguments:
  {run,record}          Command to execute:
                        - record: Start prodlog on your local machine to record commands
                        - run: Execute a command on a remote server and record its output

options:
  -h, --help           show this help message and exit
  --dir DIR, -d DIR    Base directory for logs (default: ~/.prodlog)

To use prodlog:
1. Start prodlog in record mode on your local machine:
   prodlog record

2. On remote servers, use prodlog run to execute commands:
   prodlog run <command>

Prodlog will automatically capture the output of commands run with 'prodlog run'
and save them to your local machine's log directory.
```

```
usage: prodlog_ui [-h] [--dir DIR]

Prodlog Web UI

options:
  -h, --help           show this help message and exit
  --dir DIR, -d DIR    Base directory for logs (default: ~/.prodlog)
```

## Web Interface

The web interface allows you to browse and search through your recorded sessions.

### Installation

First, install the required Python packages:

```bash
pip install -r requirements.txt
```

### Usage

Start the web interface:

```bash
prodlog_ui
```

This will start a web server on http://localhost:5000 where you can:
- Browse all recorded sessions
- Filter by date, host, and command
- Search within command output
- View formatted output with original colors and formatting
- Toggle between table and list views

You can also specify a different log directory:

```bash
prodlog_ui --dir /path/to/logs
```
