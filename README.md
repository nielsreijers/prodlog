## Quick Install

```bash
mkdir -p ~/.prodlog && curl -s https://raw.githubusercontent.com/nielsreijers/prodlog/main/prodlog > ~/.prodlog/prodlog && chmod +x ~/.prodlog/prodlog && ln -sf ~/.prodlog/prodlog ~/.local/bin/prodlog
```

Note: 
- Make sure `~/.local/bin` is in your PATH. If not, add `export PATH="$HOME/.local/bin:$PATH"` to your `~/.bashrc` or `~/.zshrc`
- No sudo required - installs only for current user

## Usage

```
usage: prodlog [-h] [--dir DIR] {run,record} ...

Record shell output

positional arguments:
  {run,record}          Command to execute

options:
  -h, --help           show this help message and exit
  --dir DIR, -d DIR    Base directory for logs (default: ~/.prodlog)
```
