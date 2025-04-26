use nix::sys::wait::waitpid;
use std::fs::File;
use std::io::{ Read, Stdout, Write };
use std::os::fd::{ AsRawFd, RawFd };
use termion::raw::{ IntoRawMode, RawTerminal };
use termion::input::TermReadEventsAndRaw;
use nix::pty::{ ForkptyResult, Winsize };
use nix::ioctl_write_ptr_bad;
use termion::terminal_size;
use tokio::sync::mpsc;
use tokio::signal::unix::{ signal, SignalKind };
use nix::unistd::execvp;
use std::ffi::CString;
use chrono::{DateTime, Utc};
use termion::{color, style};
use serde::{Serialize, Deserialize};
use std::fs;
use clap::Parser;
use std::path::PathBuf; // Use PathBuf for paths

mod ui;

const PRODLOG_CMD_PREFIX: &[u8] = "\x1A(dd0d3038-1d43-11f0-9761-022486cd4c38) PRODLOG:".as_bytes();
const CMD_IS_INACTIVE: &str = "IS CURRENTLY INACTIVE";
const CMD_ARE_YOU_RUNNING: &str = "PRODLOG, ARE YOU RUNNING?";
const CMD_START_CAPTURE: &str = "START CAPTURE";
const CMD_STOP_CAPTURE: &str = "STOP CAPTURE";
const REPLY_YES_PRODLOG_IS_RUNNING: &[u8] = "PRODLOG IS RUNNING\n".as_bytes();

/// Your application's description (optional but good practice)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Add metadata
struct CliArgs {
    /// Sets the directory for production logs.
    #[arg(long, value_name = "DIR", default_value = "/home/niels/tmp/prodlog")]
    dir: PathBuf,

    /// Sets the port for the web UI.
    #[arg(long, value_name = "PORT", default_value = "5000")]
    port: u16,

    // Add other command-line arguments here if needed
}

#[derive(Serialize, Deserialize)]
struct ProdlogEntry {
    start_time: String,
    host: String,
    command: String,
    end_time: String,
    duration_ms: u64,
    log_filename: String,
    prodlog_version: String,
}

#[derive(Serialize, Deserialize)]
struct ProdlogData {
    entries: Vec<ProdlogEntry>,
}

enum StreamState {
    InProgress(String),
    Completed(String, usize)
}

struct CaptureState {
    host: String,
    cwd: String,
    cmd: String,
    log_filename_by_host: String,
    log_by_host: File,
    log_all_hosts: File,
    start_time: DateTime<Utc>,
}

enum StdoutHandlerState {
    Normal,
    MatchingPrefix(usize),
    ReadingProdlogCommand(StreamState),
    InitCaptureHost(StreamState),
    InitCaptureCwd(String, StreamState),
    InitCaptureCmd(String, String, StreamState),
}
struct StdoutHandler {
    prodlog_dir: PathBuf,
    stdout: RawTerminal<Stdout>,
    child_stdin_tx: mpsc::Sender<Vec<u8>>,
    capturing: Option<CaptureState>,
    state: StdoutHandlerState,
}

// TODO unify these different ways of printing messages
fn print_prodlog_message(msg: &str) {
    println!("{}\n\r", format_prodlog_message(msg));
}

fn format_prodlog_message(msg: &str) -> String {
    format!("{}{}{}{}{}{}",
            style::Bold,
            color::Fg(color::Green),
            style::Blink,
            "PRODLOG: ",
            style::Reset,
            msg)
}

impl StdoutHandler {
    fn new(prodlog_dir: PathBuf, child_stdin_tx: mpsc::Sender<Vec<u8>>, stdout: RawTerminal<Stdout>) -> Self {
        Self { prodlog_dir, child_stdin_tx, stdout, capturing: None, state: StdoutHandlerState::Normal }
    }

    fn write_prodlog_message(out: &mut RawTerminal<Stdout>, msg: &str) -> Result<(), std::io::Error> {
        write!(out, "{}", format_prodlog_message(msg))?;
        out.flush()?;
        Ok(())
    }

    fn write_and_flush(&mut self, buf: &[u8]) -> Result<(), std::io::Error>  {
        self.stdout.write(buf)?;
        self.stdout.flush()?;
        if let Some(capture) = &mut self.capturing {
            capture.log_by_host.write_all(buf)?;
            capture.log_by_host.flush()?;
            capture.log_all_hosts.write_all(buf)?;
            capture.log_all_hosts.flush()?;
        }
        Ok(())
    }

    fn get_short_command(cmd: &str) -> String {
        cmd.split_whitespace().next().unwrap().to_string()
    }

    fn base64_decode(&self, data: &str) -> String {
        use base64::{Engine as _, engine::{general_purpose}};
        match general_purpose::STANDARD.decode(data) {
            Ok(bytes) => String::from_utf8(bytes).unwrap(),
            Err(e) => {
                println!("Error decoding base64: {}", e);
                data.to_string() // Shouldn't happen, but if it does, just return the original string.
            }
        }
    }

    fn get_all_hosts_log_filename (start_time: DateTime<Utc>, host: &str, cmd: &str) -> String {
        let formatted_time = start_time.format("%Y%m%d_%H%M%S").to_string();
        let short_cmd = Self::get_short_command(cmd).replace(" ", "_");
        format!("prodlog_output/all-hosts/{}-{}-{}.md", formatted_time, host, short_cmd)
    }

    fn get_by_host_log_filename (start_time: DateTime<Utc>, host: &str, cmd: &str) -> String {
        let formatted_time = start_time.format("%Y%m%d_%H%M%S").to_string();
        let short_cmd = Self::get_short_command(cmd).replace(" ", "_");
        format!("prodlog_output/{}/{}-{}.md", host, formatted_time, short_cmd)
    }

    fn start_capturing(prodlog_dir: &PathBuf, host: &str, cwd: &str, cmd: &str) -> Result<CaptureState, std::io::Error> {
        std::fs::create_dir_all(prodlog_dir.join(format!("prodlog_output/{}", host)))?;
        std::fs::create_dir_all(prodlog_dir.join("prodlog_output/all-hosts"))?;
        
        let start_time = Utc::now();
        let formatted_start_long = start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_filename_by_host = Self::get_by_host_log_filename(start_time, host, cmd);
        let log_filename_all_hosts = Self::get_all_hosts_log_filename(start_time, host, cmd);
        let mut log_by_host = File::create(prodlog_dir.join(log_filename_by_host.clone()))?;
        let mut log_all_hosts = File::create(prodlog_dir.join(log_filename_all_hosts.clone()))?;

        let header = format!(
            "Host:     {host}\n\
            Start:    {formatted_start_long}\n\
            Command:  {cmd}\n\
            Output:\n\
            ```\n\
            ");
        log_by_host.write_all(header.as_bytes())?;
        log_all_hosts.write_all(header.as_bytes())?;

        Ok(CaptureState {
            host: host.to_string(),
            cwd: cwd.to_string(),
            cmd: cmd.to_string(),
            log_filename_by_host,
            log_by_host,
            log_all_hosts,
            start_time,
        })
    }

    fn stop_capturing(prodlog_dir: &PathBuf, state: &mut CaptureState) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(prodlog_dir).unwrap();

        let end_time = Utc::now();
        let duration_ms = end_time.signed_duration_since(state.start_time).num_milliseconds() as u64;
        let formatted_start_short = state.start_time.format("%Y-%m-%d %H:%M");
        let formatted_start_long = state.start_time.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let formatted_end_long = end_time.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let host = &state.host;
        let cmd_short = Self::get_short_command(&state.cmd);
        let cmd_long = &state.cmd;
        let log_filename = &state.log_filename_by_host;

        let entry = format!(
            "\n## {formatted_start_short} on {host}: {cmd_short}\n\
            ```\n\
            Host:     {host}\n\
            Start:    {formatted_start_long}\n\
            Command:  {cmd_long}\n\
            End:      {formatted_end_long}\n\
            Duration: {duration_ms}ms\n\
            ```\n\
            Output:   [[{log_filename}]]\n\
            \n\
            ---\n\
            ");

        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(prodlog_dir.join("prodlog.md"))?
            .write_all(entry.as_bytes())?;

        // Log to JSON file for webui
        let json_path = prodlog_dir.join("prodlog.json");
        let mut prodlog_data = if let Ok(content) = fs::read_to_string(&json_path) {
            serde_json::from_str(&content).unwrap_or(ProdlogData { entries: Vec::new() })
        } else {
            ProdlogData { entries: Vec::new() }
        };
        prodlog_data.entries.push(ProdlogEntry {
            host: host.to_string(),
            start_time: state.start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            duration_ms,
            command: cmd_long.to_string(),
            log_filename: log_filename.to_string(),
            prodlog_version: env!("CARGO_PKG_VERSION").to_string(),
        });
        fs::write(&json_path, serde_json::to_string_pretty(&prodlog_data)?)?;


        let footer = format!(
            "```\n\
            End:      {formatted_end_long}\n\
            Duration: {duration_ms}ms\n");
        state.log_by_host.write_all(footer.as_bytes())?;
        state.log_all_hosts.write_all(footer.as_bytes())?;
        state.log_by_host.flush()?;
        state.log_all_hosts.flush()?;

        Ok(())
    }

    fn read_until_terminator(&self, buffer: &[u8], mut pos: usize, n: usize, state: &StreamState) -> StreamState {
        if let StreamState::InProgress(partial) = state {
            let start= pos;
            while pos < n && buffer[pos] != b';' {
                pos += 1;
            }
            let new_value = partial.to_owned() + &String::from_utf8_lossy(&buffer[start..pos]).to_string();
            if pos == n {
                // Ran out of data, wait for next chunk
                StreamState::InProgress(new_value)
            } else {
                // Skip terminator
                pos += 1;
                StreamState::Completed(new_value, pos)
            }
        } else {
            panic!("Invalid state");
        }
    }

    fn process(&mut self, buffer: &[u8], n: usize) -> Result<(), std::io::Error> {
        let mut pos = 0;
        while pos < n {
            match &self.state {
                StdoutHandlerState::Normal => {
                    let start = pos;
                    while pos < n && buffer[pos] != PRODLOG_CMD_PREFIX[0] {
                        pos += 1;
                    }
                    // Print the data scanned so far
                    self.write_and_flush(&buffer[start..pos])?;
                    if pos < n {
                        // If pos < n, the current character may be the start of a PRODLOG command.
                        self.state = StdoutHandlerState::MatchingPrefix(1);
                        pos += 1;
                    }
                }
                StdoutHandlerState::MatchingPrefix(mut bytes_matched) => {
                    while
                        pos < n &&
                        bytes_matched < PRODLOG_CMD_PREFIX.len() &&
                        buffer[pos] == PRODLOG_CMD_PREFIX[bytes_matched]
                    {
                        pos += 1;
                        bytes_matched += 1;
                    }
                    if bytes_matched == PRODLOG_CMD_PREFIX.len() {
                        // Command prefix matched, start reading the command.
                        self.state = StdoutHandlerState::ReadingProdlogCommand(StreamState::InProgress("".to_string()));
                    } else if pos == n {
                        // Ran out of data, wait for next chunk
                        self.state = StdoutHandlerState::MatchingPrefix(bytes_matched);
                    } else {
                        // Prefix didn't match, print the bytes that did match
                        self.write_and_flush(&PRODLOG_CMD_PREFIX[..bytes_matched])?;
                        // And reset the state to Normal
                        self.state = StdoutHandlerState::Normal;
                    }
                }
                StdoutHandlerState::ReadingProdlogCommand(stream_state) => {
                    let stream_state = self.read_until_terminator(buffer, pos, n, &stream_state);
                    match stream_state {
                        StreamState::InProgress(_) => {
                            self.state = StdoutHandlerState::ReadingProdlogCommand(stream_state);
                            pos = n;
                        }
                        StreamState::Completed(cmd, new_pos) => {
                            pos = new_pos;
                            match cmd.as_str() {
                                CMD_IS_INACTIVE => {
                                    Self::write_prodlog_message(&mut self.stdout, "Prodlog is currently active!")?;
                                    self.state = StdoutHandlerState::Normal;
                                }
                                CMD_ARE_YOU_RUNNING => {
                                    // TODO: figure out why async send doesn't work here. It works fine in run_parent. Are we deadlocking?
                                    Self::write_prodlog_message(&mut self.stdout, "Telling server side prodlog recording is active:")?;
                                    self.child_stdin_tx.blocking_send(REPLY_YES_PRODLOG_IS_RUNNING.to_vec()).unwrap();
                                    self.state = StdoutHandlerState::Normal;
                                }
                                CMD_START_CAPTURE => {
                                    self.state = StdoutHandlerState::InitCaptureHost(StreamState::InProgress("".to_string()));
                                }
                                CMD_STOP_CAPTURE => {
                                    if let Some(capture) = &mut self.capturing {
                                        Self::write_prodlog_message(&mut self.stdout, &format!("Stopping capture of {} on {}:{}",
                                                                            capture.cmd,
                                                                            capture.host,
                                                                            capture.cwd))?;
                                        Self::stop_capturing(&self.prodlog_dir, capture)?;
                                    } else {
                                        Self::write_prodlog_message(&mut self.stdout, "Warning: Tried to stop capture, but no capture was active")?
                                    }
                                    self.capturing = None;
                                    self.state = StdoutHandlerState::Normal;
                                }
                                _ => {
                                    // Unknown command. Just print what we saw on the child's stdout.
                                    self.write_and_flush(PRODLOG_CMD_PREFIX)?;
                                    self.write_and_flush(cmd.as_bytes())?;
                                    self.write_and_flush(b";")?;
                                    self.state = StdoutHandlerState::Normal;
                                }
                            }
                        }
                    }
                },
                StdoutHandlerState::InitCaptureHost(stream_state) => {
                    let stream_state = self.read_until_terminator(buffer, pos, n, &stream_state);
                    match stream_state {
                        StreamState::InProgress(_) => {
                            self.state = StdoutHandlerState::InitCaptureHost(stream_state);
                            pos = n;
                        }
                        StreamState::Completed(host, new_pos) => {
                            let host = self.base64_decode(&host);
                            self.state = StdoutHandlerState::InitCaptureCwd(host, StreamState::InProgress("".to_string()));
                            pos = new_pos;
                        }
                    }
                }
                StdoutHandlerState::InitCaptureCwd(host, stream_state) => {
                    let stream_state = self.read_until_terminator(buffer, pos, n, &stream_state);
                    match stream_state {
                        StreamState::InProgress(_) => {
                            self.state = StdoutHandlerState::InitCaptureCwd(host.to_string(), stream_state);
                            pos = n;
                        }
                        StreamState::Completed(cwd, new_pos) => {
                            let cwd = self.base64_decode(&cwd);
                            self.state = StdoutHandlerState::InitCaptureCmd(host.to_string(), cwd, StreamState::InProgress("".to_string()));
                            pos = new_pos;
                        }
                    }
                }
                StdoutHandlerState::InitCaptureCmd(host, cwd, stream_state) => {
                    let stream_state = self.read_until_terminator(buffer, pos, n, &stream_state);
                    match stream_state {
                        StreamState::InProgress(_) => {
                            self.state = StdoutHandlerState::InitCaptureCmd(host.to_string(), cwd.to_string(), stream_state);
                            pos = n;
                        }
                        StreamState::Completed(cmd, new_pos) => {
                            let cmd = self.base64_decode(&cmd);
                            Self::write_prodlog_message(&mut self.stdout, &format!("Starting capture of {} on {}:{}", cmd, host, cwd))?;
                            self.capturing = Some(Self::start_capturing(&self.prodlog_dir, host, cwd, &cmd)?);
                            self.state = StdoutHandlerState::Normal;
                            pos = new_pos;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn run_child() -> Result<(), std::io::Error> {
    // let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    let shell = String::from("/bin/bash");
    let cmd = CString::new(shell).expect("CString::new failed");
    let args: [CString; 0] = [];
    execvp(&cmd, &args)?;
    Ok(())
}

fn set_winsize(fd: RawFd) -> Result<(), std::io::Error> {
    // Get the current terminal size
    let (cols, rows) = terminal_size()?;
    let winsize = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
    unsafe {
        ioctl_write_ptr_bad!(tiocswinsz, libc::TIOCSWINSZ, Winsize);
        tiocswinsz(fd, &winsize as *const _ as *mut _)?;
    }
    Ok(())
}

async fn run_parent(
    cli_args: &CliArgs,
    child: nix::unistd::Pid,
    master: std::os::fd::OwnedFd
) -> Result<(), std::io::Error> {
    // Set terminal to raw mode
    let raw_stdout = std::io::stdout().into_raw_mode()?;

    // Get the file descriptor for the master pty.
    let master_fd = master.as_raw_fd();

    // Get the File handles to communicate with the child.
    let mut raw_master_write = File::from(master);
    let mut raw_master_read = raw_master_write.try_clone()?;

    // Create a channel for sending to the child's stdin
    let (child_stdin_tx, mut child_stdin_rx) = mpsc::channel::<Vec<u8>>(100);
    let child_stdin_tx2 = child_stdin_tx.clone();

    // Forward whatever bytes appear on the channel to the child's stdin.
    let _stdin_sender_handle = tokio::spawn(async move {
        while let Some(data) = child_stdin_rx.recv().await {
            raw_master_write.write(&data).unwrap();
            raw_master_write.flush().unwrap();
        }
    });

    // Read our stdin and forward it to the child.
    let _stdin_reader_thread = tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();
        for event in stdin.events_and_raw() {
            let (_, raw) = event.unwrap();
            if child_stdin_tx.blocking_send(raw).is_err() {
                eprintln!("Input thread: Tokio receiver dropped.");
                break; // Exit the thread
            }
        }
    });

    // Start forwarding the child's stdout to our stdout.
    let prodlog_dir = cli_args.dir.clone();
    let _forward_stdout = tokio::task::spawn_blocking(move || {
        let mut buffer = [0; 1024];
        let mut stream_handler = StdoutHandler::new(prodlog_dir, child_stdin_tx2, raw_stdout);
        loop {
            let n = raw_master_read.read(&mut buffer);
            if let Ok(n) = n {
                if n == 0 {
                    break; // EOF reached
                }
                stream_handler.process(&buffer, n).unwrap();
            } else {
                break;
            }
        }
    });

    // Start listening for window size changes and forward them to the child.
    let _winsize_listener = tokio::spawn(async move {
        let mut sigwinch_stream = signal(SignalKind::window_change()).unwrap();
        set_winsize(master_fd).unwrap();
        loop {
            sigwinch_stream.recv().await;
            set_winsize(master_fd).unwrap();
        }
    });

    // Wait for the child to exit.
    let wait_child_exit = tokio::task::spawn_blocking(move || {
        waitpid(child, None).unwrap();
    });
    wait_child_exit.await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() {
    let cli_args = CliArgs::parse();
    println!("prodlog logging to {:?}", cli_args.dir);

    // Create the directory if it doesn't exist
    fs::create_dir_all(&cli_args.dir).expect("Failed to create directory");
    
    // Start the UI in a separate task
    let ui_dir = cli_args.dir.clone();
    let ui_port = cli_args.port;
    tokio::spawn(async move {
        ui::run_ui(&ui_dir, ui_port).await;
    });

    let result = match (unsafe { nix::pty::forkpty(None, None) }).unwrap() {
        ForkptyResult::Child => run_child(),
        ForkptyResult::Parent { child, master } => { run_parent(&cli_args, child, master).await }
    };
    if let Err(e) = result {
        eprintln!("PRODLOG EXITING WITH ERROR: {}", e);
        std::process::exit(1);
    } else {
        println!("PRODLOG EXITING");
        std::process::exit(0);
    }
}