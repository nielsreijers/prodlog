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
use std::fs;
use clap::Parser;
use std::path::PathBuf; // Use PathBuf for paths
use dirs;
use uuid::Uuid; // Add uuid dependency

mod ui;
mod sinks;

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
    #[arg(long, value_name = "DIR", default_value = ".local/share/prodlog")]
    dir: PathBuf,

    /// Sets the port for the web UI.
    #[arg(long, value_name = "PORT", default_value = "5000")]
    port: u16,

    // Add other command-line arguments here if needed
}

enum StreamState {
    InProgress(String),
    Completed(String, Vec<String>, usize)
}

struct CaptureState {
    uuid: Uuid,
    host: String,
    cwd: String,
    cmd: String,
    start_time: DateTime<Utc>,
    captured_output: Vec<u8>,
}

enum StdoutHandlerState {
    Normal,
    MatchingPrefix(usize),
    ReadingProdlogCommand(StreamState),
}
struct StdoutHandler {
    stdout: RawTerminal<Stdout>,
    child_stdin_tx: mpsc::Sender<Vec<u8>>,
    capturing: Option<CaptureState>,
    state: StdoutHandlerState,
    sinks: Vec<Box<dyn sinks::Sink>>,
}

// TODO unify these different ways of printing messages
fn print_prodlog_message(msg: &str) {
    print!("{}\n\r", format!("{}{}{}{}{}{}",
        style::Bold,
        color::Fg(color::Green),
        style::Blink,
        "PRODLOG: ",
        style::Reset,
        msg));
}

impl StdoutHandler {
    fn new(child_stdin_tx: mpsc::Sender<Vec<u8>>, stdout: RawTerminal<Stdout>, sinks: Vec<Box<dyn sinks::Sink>>) -> Self {
        Self { child_stdin_tx, stdout, capturing: None, state: StdoutHandlerState::Normal, sinks }
    }

    fn write_and_flush(&mut self, buf: &[u8]) -> Result<(), std::io::Error>  {
        self.stdout.write(buf)?;
        self.stdout.flush()?;
        if let Some(capture) = &mut self.capturing {
            capture.captured_output.extend_from_slice(buf);
        }
        Ok(())
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


    fn start_capturing(host: &str, cwd: &str, cmd: &str) -> Result<CaptureState, std::io::Error> {
        let start_time = Utc::now();

        Ok(CaptureState {
            uuid: Uuid::new_v4(),
            host: host.to_string(),
            cwd: cwd.to_string(),
            cmd: cmd.to_string(),
            start_time,
            captured_output: Vec::new(),
        })
    }

    fn stop_capturing(state: &mut CaptureState, exit_code: i32, sinks: &mut Vec<Box<dyn sinks::Sink>>) -> Result<(), std::io::Error> {
        for sink in sinks {
            match sink.add_entry(state, exit_code, Utc::now()) {
                Ok(_) => (),
                Err(e) => print_prodlog_message(&format!("Error writing to sink: {}", e)),
            }
        }

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
                let mut split = new_value.splitn(2, ':');
                let cmd = split.next().unwrap_or("").to_string();
                let rest = split.next().unwrap_or("");
                let args: Vec<String> = if rest.is_empty() {
                    Vec::new()
                } else {
                    rest.split(':').map(|s| self.base64_decode(s)).collect()
                };
                StreamState::Completed(cmd, args, pos)
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
                        StreamState::Completed(cmd, args, new_pos) => {
                            pos = new_pos;
                            match cmd.as_str() {
                                CMD_IS_INACTIVE => {
                                    print_prodlog_message("Prodlog is currently active!");
                                    self.state = StdoutHandlerState::Normal;
                                }
                                CMD_ARE_YOU_RUNNING => {
                                    // TODO: figure out why async send doesn't work here. It works fine in run_parent. Are we deadlocking?
                                    print_prodlog_message("Telling server side prodlog recording is active:");
                                    self.child_stdin_tx.blocking_send(REPLY_YES_PRODLOG_IS_RUNNING.to_vec()).unwrap();
                                    self.state = StdoutHandlerState::Normal;
                                }
                                CMD_START_CAPTURE => {
                                    // TODO: error handling
                                    if let (Some(client_host), Some(client_cwd), Some(client_cmd)) = (args.get(0), args.get(1), args.get(2)) {
                                        print_prodlog_message(&format!("Starting capture of {} on {}:{}", client_cmd, client_host, client_cwd));
                                        self.capturing = Some(Self::start_capturing(client_host, client_cwd, client_cmd)?);
                                        self.state = StdoutHandlerState::Normal;
                                        pos = new_pos;
                                    } else {
                                        print_prodlog_message("Error: Missing arguments for START CAPTURE");
                                        self.state = StdoutHandlerState::Normal;
                                    }
                                },
                                CMD_STOP_CAPTURE => {
                                    let exit_code = args.get(0)
                                        .and_then(|s| s.parse::<i32>().ok())
                                        .unwrap_or(1000);
                                    if let Some(capture) = &mut self.capturing {
                                        print_prodlog_message(&format!("Stopping capture of {} on {}:{} with exit code {}",
                                                                            capture.cmd,
                                                                            capture.host,
                                                                            capture.cwd,
                                                                            exit_code));
                                        Self::stop_capturing(capture, exit_code, &mut self.sinks)?;
                                    } else {
                                        print_prodlog_message("Warning: Tried to stop capture, but no capture was active");
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
    prodlog_dir: &PathBuf,
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
    let prodlog_dir = prodlog_dir.clone();
    let _forward_stdout = tokio::task::spawn_blocking(move || {
        let mut buffer = [0; 1024];
        let sinks: Vec<Box<dyn sinks::Sink>> = vec![
            Box::new(sinks::json::JsonSink::new(prodlog_dir.clone())),
            Box::new(sinks::obsidian::ObsidianSink::new(prodlog_dir.clone())),
            Box::new(sinks::sqlite::SqliteSink::new(prodlog_dir.clone()).unwrap()),
        ];
        let mut stream_handler = StdoutHandler::new(child_stdin_tx2, raw_stdout, sinks);
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
    
    // Get the log directory path
    let prodlog_dir = if cli_args.dir.is_absolute() {
        cli_args.dir.clone()
    } else {
        // For relative paths, prepend the home directory
        let home_dir = dirs::home_dir().expect("Could not determine home directory");
        home_dir.join(&cli_args.dir)
    };
    print_prodlog_message(&format!("prodlog logging to {:?}", prodlog_dir));

    
    // Create the directory if it doesn't exist
    fs::create_dir_all(&prodlog_dir).expect("Failed to create directory");
    
    // Start the UI in a separate task
    let ui_dir = prodlog_dir.clone();
    let ui_port = cli_args.port;
    tokio::spawn(async move {
        ui::run_ui(&ui_dir, ui_port).await;
    });

    let result = match (unsafe { nix::pty::forkpty(None, None) }).unwrap() {
        ForkptyResult::Child => run_child(),
        ForkptyResult::Parent { child, master } => { run_parent(&prodlog_dir, child, master).await }
    };
    if let Err(e) = result {
        print_prodlog_message(&format!("PRODLOG EXITING WITH ERROR: {}", e));
        std::process::exit(1);
    } else {
        print_prodlog_message("PRODLOG EXITING");
        std::process::exit(0);
    }
}