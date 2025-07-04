use nix::sys::wait::waitpid;
use sinks::UiSource;
use termion::color::Color;
use std::fs::File;
use std::io::{ Read, Stdout, Write };
use std::os::fd::{ AsRawFd, RawFd };
use std::sync::Arc;
use termion::raw::{ IntoRawMode, RawTerminal };
use termion::input::TermReadEventsAndRaw;
use nix::pty::{ ForkptyResult, Winsize };
use nix::ioctl_write_ptr_bad;
use termion::terminal_size;
use tokio::sync::{ mpsc, RwLock };
use tokio::signal::unix::{ signal, SignalKind };
use nix::unistd::execvp;
use std::ffi::CString;
use chrono::Utc;
use termion::{ color, style };
use std::fs;
use std::path::PathBuf;
use dirs;
use uuid::Uuid;
use whoami;
use model::{ CaptureType, CaptureV2_4 };

use crate::config::get_config;

mod ui;
mod sinks;
mod helpers;
mod config;
mod model;

const PRODLOG_CMD_PREFIX: &[u8] = "\x1A(dd0d3038-1d43-11f0-9761-022486cd4c38) PRODLOG:".as_bytes();
const CMD_IS_INACTIVE: &str = "IS CURRENTLY INACTIVE";
const CMD_ARE_YOU_RUNNING: &str = "PRODLOG, ARE YOU RUNNING?";
const CMD_START_CAPTURE_RUN: &str = "START CAPTURE RUN";
const CMD_START_CAPTURE_EDIT: &str = "START CAPTURE EDIT";
const CMD_STOP_CAPTURE_RUN: &str = "STOP CAPTURE RUN";
const CMD_STOP_CAPTURE_EDIT: &str = "STOP CAPTURE EDIT";
const REPLY_YES_PRODLOG_IS_RUNNING: &[u8] = "PRODLOG IS RUNNING\n".as_bytes();

enum StreamState {
    InProgress(String),
    Completed(String, Vec<String>, usize),
}

enum StdoutHandlerState {
    Normal,
    MatchingPrefix(usize),
    ReadingProdlogCommand(StreamState),
}
struct StdoutHandler {
    stdout: RawTerminal<Stdout>,
    child_stdin_tx: mpsc::Sender<Vec<u8>>,
    capturing: Option<CaptureV2_4>,
    state: StdoutHandlerState,
    sinks: Vec<Box<dyn sinks::Sink>>,
}

// TODO unify these different ways of printing messages
fn prodlog_print<C: Color>(msg: &str, color: C) {
    print!(
        "{}\n\r",
        format!(
            "{}{}{}{}{}{}",
            style::Bold,
            color::Fg(color),
            style::Blink,
            "PRODLOG: ",
            style::Reset,
            msg
        )
    );
}

fn prodlog_panic(msg: &str) -> ! {
    prodlog_print(msg, color::Red);
    std::process::exit(1);
}

pub fn print_prodlog_warning(msg: &str) {
    prodlog_print(msg, color::Yellow);
}

fn print_prodlog_message(msg: &str) {
    prodlog_print(msg, color::Green);
}

impl StdoutHandler {
    fn new(
        child_stdin_tx: mpsc::Sender<Vec<u8>>,
        stdout: RawTerminal<Stdout>,
        sinks: Vec<Box<dyn sinks::Sink>>
    ) -> Self {
        Self { child_stdin_tx, stdout, capturing: None, state: StdoutHandlerState::Normal, sinks }
    }

    fn write_and_flush(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        self.stdout.write(buf)?;
        self.stdout.flush()?;
        if let Some(capture) = &mut self.capturing {
            capture.captured_output.extend_from_slice(buf);
        }
        Ok(())
    }

    fn start_capturing_run(
        host: &str,
        cwd: &str,
        cmd: &str,
        message: &str,
        remote_user: &str
    ) -> Result<CaptureV2_4, std::io::Error> {
        let start_time = Utc::now();

        Ok(CaptureV2_4 {
            capture_type: CaptureType::Run,
            uuid: Uuid::new_v4(),
            host: host.to_string(),
            cwd: cwd.to_string(),
            cmd: cmd.to_string(),
            start_time,
            duration_ms: 0,
            message: message.to_string(),
            is_noop: false,
            exit_code: -1,
            local_user: whoami::username(),
            remote_user: remote_user.to_string(),
            filename: "".to_string(),
            terminal_rows: 0,
            terminal_cols: 0,
            captured_output: Vec::new(),
            original_content: "".as_bytes().to_vec(),
            edited_content: "".as_bytes().to_vec(),
        })
    }

    fn stop_capturing_run(
        capture: &mut CaptureV2_4,
        exit_code: i32,
        sinks: &mut Vec<Box<dyn sinks::Sink>>
    ) -> Result<(), std::io::Error> {
        capture.exit_code = exit_code;
        capture.duration_ms = Utc::now()
            .signed_duration_since(capture.start_time)
            .num_milliseconds() as u64;
        let (cols, rows) = terminal_size()?;
        capture.terminal_cols = cols;
        capture.terminal_rows = rows;
        for sink in sinks {
            match sink.add_entry(capture) {
                Ok(_) => (),
                Err(e) => print_prodlog_message(&format!("Error writing to sink: {}", e)),
            }
        }

        Ok(())
    }

    fn start_capturing_edit(
        host: &str,
        cwd: &str,
        cmd: &str,
        message: &str,
        remote_user: &str,
        filename: &str,
        original_content: Vec<u8>
    ) -> Result<CaptureV2_4, std::io::Error> {
        let start_time = Utc::now();

        Ok(CaptureV2_4 {
            capture_type: CaptureType::Edit,
            uuid: Uuid::new_v4(),
            host: host.to_string(),
            cwd: cwd.to_string(),
            cmd: cmd.to_string(),
            start_time,
            captured_output: Vec::new(),
            message: message.to_string(),
            duration_ms: 0,
            is_noop: false,
            exit_code: -1,
            local_user: whoami::username(),
            remote_user: remote_user.to_string(),
            filename: filename.to_string(),
            original_content: original_content,
            edited_content: "".as_bytes().to_vec(),
            terminal_rows: 0,
            terminal_cols: 0,
        })
    }

    fn stop_capturing_edit(
        capture: &mut CaptureV2_4,
        exit_code: i32,
        edited_content: Vec<u8>,
        sinks: &mut Vec<Box<dyn sinks::Sink>>
    ) -> Result<(), std::io::Error> {
        capture.exit_code = exit_code;
        capture.duration_ms = Utc::now()
            .signed_duration_since(capture.start_time)
            .num_milliseconds() as u64;
        capture.edited_content = edited_content;
        for sink in sinks {
            match sink.add_entry(capture) {
                Ok(_) => (),
                Err(e) => print_prodlog_message(&format!("Error writing to sink: {}", e)),
            }
        }

        Ok(())
    }

    fn read_until_terminator(
        &self,
        buffer: &[u8],
        mut pos: usize,
        n: usize,
        state: &StreamState
    ) -> StreamState {
        if let StreamState::InProgress(partial) = state {
            let start = pos;
            while pos < n && buffer[pos] != b';' {
                pos += 1;
            }
            let new_value =
                partial.to_owned() + &String::from_utf8_lossy(&buffer[start..pos]).to_string();
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
                    rest.split(':')
                        .map(|s| helpers::base64_decode_string(s))
                        .collect()
                };
                StreamState::Completed(cmd, args, pos)
            }
        } else {
            prodlog_panic("Invalid state");
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
                        self.state = StdoutHandlerState::ReadingProdlogCommand(
                            StreamState::InProgress("".to_string())
                        );
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
                                    if let Some(version) = args.get(0) {
                                        if !helpers::compare_major_minor_versions(version, env!("CARGO_PKG_VERSION")) {
                                            print_prodlog_message(
                                                &format!(
                                                    "Error: Unsupported version: {} (expected major.minor to match {})",
                                                    version,
                                                    env!("CARGO_PKG_VERSION")
                                                )
                                            );
                                        } else {
                                            print_prodlog_message(
                                                "Telling server side prodlog recording is active:"
                                            );
                                            // TODO: figure out why async send doesn't work here. It works fine in run_parent. Are we deadlocking?
                                            self.child_stdin_tx
                                                .blocking_send(
                                                    REPLY_YES_PRODLOG_IS_RUNNING.to_vec()
                                                )
                                                .unwrap();
                                        }
                                    } else {
                                        print_prodlog_message("Error: Missing version argument");
                                    }
                                    self.state = StdoutHandlerState::Normal;
                                    pos = new_pos;
                                }
                                CMD_START_CAPTURE_RUN => {
                                    // TODO: error handling
                                    if
                                        let (Some(host), Some(cwd), Some(cmd), Some(message), Some(remote_user)) = (
                                            args.get(0),
                                            args.get(1),
                                            args.get(2),
                                            args.get(3),
                                            args.get(4),
                                        )
                                    {
                                        print_prodlog_message(
                                            &format!(
                                                "Starting capture of {} on {}:{}",
                                                cmd,
                                                host,
                                                cwd
                                            )
                                        );
                                        self.capturing = Some(
                                            Self::start_capturing_run(host, cwd, cmd, message, remote_user)?
                                        );
                                        self.state = StdoutHandlerState::Normal;
                                        pos = new_pos;
                                    } else {
                                        print_prodlog_message(
                                            "Error: Missing arguments for START CAPTURE RUN"
                                        );
                                        self.state = StdoutHandlerState::Normal;
                                        pos = new_pos;
                                    }
                                }
                                CMD_STOP_CAPTURE_RUN => {
                                    let exit_code = args
                                        .get(0)
                                        .and_then(|s| s.parse::<i32>().ok())
                                        .unwrap_or(1000);
                                    if let Some(capture) = &mut self.capturing {
                                        print_prodlog_message(
                                            &format!(
                                                "Stopping capture of {} on {}:{} with exit code {}",
                                                capture.cmd,
                                                capture.host,
                                                capture.cwd,
                                                exit_code
                                            )
                                        );
                                        Self::stop_capturing_run(
                                            capture,
                                            exit_code,
                                            &mut self.sinks
                                        )?;
                                    } else {
                                        print_prodlog_message(
                                            "Warning: Tried to stop capture, but no capture was active"
                                        );
                                    }
                                    self.capturing = None;
                                    self.state = StdoutHandlerState::Normal;
                                }
                                CMD_START_CAPTURE_EDIT => {
                                    // TODO: error handling
                                    if  let (
                                            Some(host),
                                            Some(cwd),
                                            Some(cmd),
                                            Some(message),
                                            Some(remote_user),
                                            Some(filename),
                                            Some(original_content),
                                        ) = (
                                            args.get(0),
                                            args.get(1),
                                            args.get(2),
                                            args.get(3),
                                            args.get(4),
                                            args.get(5),
                                            args.get(6))
                                    {
                                        print_prodlog_message(
                                            &format!(
                                                "Starting capture of editing file {} on {}",
                                                filename,
                                                host
                                            )
                                        );
                                        let original_content =
                                            helpers::base64_decode(original_content);
                                        self.capturing = Some(
                                            Self::start_capturing_edit(
                                                host,
                                                cwd,
                                                cmd,
                                                message,
                                                remote_user,
                                                filename,
                                                original_content
                                            )?
                                        );
                                        self.state = StdoutHandlerState::Normal;
                                        pos = new_pos;
                                    } else {
                                        print_prodlog_message(
                                            "Error: Missing arguments for START CAPTURE EDIT"
                                        );
                                        self.state = StdoutHandlerState::Normal;
                                        pos = new_pos;
                                    }
                                }
                                CMD_STOP_CAPTURE_EDIT => {
                                    let empty = "".to_string();
                                    let exit_code = args
                                        .get(0)
                                        .and_then(|s| s.parse::<i32>().ok())
                                        .unwrap_or(1000);
                                    let edited_content = args.get(1).unwrap_or(&empty);
                                    let edited_content = helpers::base64_decode(edited_content);
                                    if let Some(capture) = &mut self.capturing {
                                        print_prodlog_message(
                                            &format!(
                                                "Stopping capture of editing file {} on {} with exit code {}",
                                                capture.filename,
                                                capture.host,
                                                exit_code
                                            )
                                        );
                                        Self::stop_capturing_edit(
                                            capture,
                                            exit_code,
                                            edited_content,
                                            &mut self.sinks
                                        )?;
                                    } else {
                                        print_prodlog_message(
                                            "Warning: Tried to stop capture, but no capture was active"
                                        );
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
                }
            }
        }
        Ok(())
    }
}

fn run_child() -> Result<(), std::io::Error> {
    let cmdline = get_config().cmd.clone();
    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    if parts.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Empty shell command"));
    }
    let args: Vec<CString> = parts
        .iter()
        .map(|&s| CString::new(s).expect("CString::new failed"))
        .collect();    
    execvp(&args[0], &args)?;
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

fn get_sinks(prodlog_dir: &PathBuf) -> Vec<Box<dyn sinks::Sink>> {
    fs::create_dir_all(prodlog_dir).expect("Failed to create directory");
    let json_file = prodlog_dir.join("prodlog.json");
    let sqlite_file = prodlog_dir.join("prodlog.sqlite");

    vec![
        Box::new(sinks::obsidian::ObsidianSink::new(&prodlog_dir)),
        Box::new(sinks::json::JsonSink::new(&json_file)),
        Box::new(sinks::sqlite::SqliteSink::new(&sqlite_file))
    ]
}

async fn run_parent(
    sinks: Vec<Box<dyn sinks::Sink>>,
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
    let _forward_stdout = tokio::task::spawn_blocking(move || {
        let mut buffer = [0; 1024];
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

fn import(import_file: &str, sinks: &mut Vec<Box<dyn sinks::Sink>>) -> Result<(), std::io::Error> {
    let import_file = PathBuf::from(import_file);

    if !import_file.exists() {
        prodlog_panic(&format!("Error: Import file {:?} does not exist", import_file));
    }

    print_prodlog_message(&format!("Importing from {:?}", import_file));
    let source_sink: Box<dyn sinks::UiSource> = match
        import_file.extension().unwrap_or_default().to_str().unwrap_or_default()
    {
        "json" => { Box::new(sinks::json::JsonSink::new(&import_file)) }
        "sqlite" => {
            // TODO: copy to tmp file so we don't modify the original if the schema changed.
            Box::new(sinks::sqlite::SqliteSink::new(&import_file))
        }
        _ => {
            prodlog_panic(
                &format!("Error: Import file must be .json or .sqlite, got {:?}", import_file)
            );
        }
    };

    // Import all entries from source
    let entries = source_sink.get_entries(&sinks::Filters::default())?;
    print_prodlog_message(&format!("Found {} entries to import", entries.len()));
    for entry in entries {
        for sink in sinks.iter_mut() {
            if let Err(e) = sink.add_entry(&entry) {
                print_prodlog_warning(
                    &format!("Error writing entry {} to sink: {}", entry.uuid, e)
                );
            }
        }
    }
    print_prodlog_message("Import done.");

    Ok(())
}

#[tokio::main]
async fn main() {
    // Get the log directory path
    let prodlog_dir = if get_config().dir.is_absolute() {
        get_config().dir.clone()
    } else {
        // For relative paths, prepend the home directory
        let home_dir = dirs::home_dir().expect("Could not determine home directory");
        home_dir.join(get_config().dir.clone())
    };
    print_prodlog_message(&format!("prodlog logging to {:?}", prodlog_dir));

    // Create the directory doesn't exist
    let mut sinks = get_sinks(&prodlog_dir);

    // Import a prodlog json or sqlite file if specified2
    if let Some(import_file) = &get_config().import {
        import(&import_file, &mut sinks).unwrap();
    }

    // Start the UI in a separate task
    let ui_port = get_config().port;
    tokio::spawn(async move {
        // let sink = Arc::new(sinks::json::JsonSink::new(ui_dir));
        let sqlite_file = prodlog_dir.join("prodlog.sqlite");
        let sink: Arc<RwLock<Box<dyn UiSource>>> = Arc::new(
            RwLock::new(Box::new(sinks::sqlite::SqliteSink::new(&sqlite_file)))
        );
        ui::run_ui(sink, ui_port).await;
    });

    let mut is_child = "";
    let result = match (unsafe { nix::pty::forkpty(None, None) }).unwrap() {
        ForkptyResult::Child => {
            is_child = "CHILD PROCESS ";
            run_child()
        },
        ForkptyResult::Parent { child, master } => { run_parent(sinks, child, master).await }
    };
    if let Err(e) = result {
        prodlog_panic(&format!("PRODLOG {}EXITING WITH ERROR: {}", is_child, e));
    } else {
        print_prodlog_message(&format!("PRODLOG {}EXITING", is_child));
        std::process::exit(0);
    }
}
