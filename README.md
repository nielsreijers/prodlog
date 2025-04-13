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

Prodlog includes a web interface to browse and search your command history. To start it:

```bash
prodlog_ui
```

This will start a local web server at http://localhost:5000. Open this URL in your browser to access the interface.

The web interface allows you to:
- View all recorded commands in chronological order
- Filter commands by date, hostname, or command text
- View the full output of any recorded command
- Navigate between different hosts and sessions

You can also specify a different log directory:

```bash
prodlog_ui --dir /path/to/logs
```
