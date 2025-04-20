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

const PRODLOG_CMD_PREFIX: &[u8] = "#### PRODLOG(dd0d3038-1d43-11f0-9761-022486cd4c38):".as_bytes();

enum StreamState {
    InProgress(String),
    Completed(String, usize)
}

struct CaptureState {
    host: String,
    cwd: String,
    cmd: String,
    log: File,
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
    stdout: RawTerminal<Stdout>,
    capturing: Option<CaptureState>,
    state: StdoutHandlerState,
}

impl StdoutHandler {
    fn new(stdout: RawTerminal<Stdout>) -> Self {
        Self { stdout, capturing: None, state: StdoutHandlerState::Normal }
    }

    fn write_prodlog_message(out: &mut RawTerminal<Stdout>, msg: &str) -> Result<(), std::io::Error> {

        out.write(b"PRODLOG: ");
        out.write(msg.as_bytes());
        out.write(b"\n\r");
        out.flush();
        Ok(())
    }

    fn write_and_flush(&mut self, buf: &[u8]) -> Result<(), std::io::Error>  {
        self.stdout.write(buf)?;
        self.stdout.flush()?;
        if let Some(capture) = &mut self.capturing {
            capture.log.write_all(buf)?;
            capture.log.flush()?;
        }
        Ok(())
    }

    fn start_capturing(host: &str, cwd: &str, cmd: &str) -> CaptureState {
        CaptureState {
            host: host.to_string(),
            cwd: cwd.to_string(),
            cmd: cmd.to_string(),
            log: File::create(format!("/home/niels/tmp/prodlog/{}.log", Utc::now().format("%Y-%m-%d_%H-%M-%S"))).unwrap(),
            start_time: Utc::now(),
        }
    }

    fn stop_capturing(state: &CaptureState) {
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
                                "IS CURRENTLY INACTIVE" => {
                                    Self::write_prodlog_message(&mut self.stdout, "Prodlog is currently active!")?;
                                    self.state = StdoutHandlerState::Normal;
                                }
                                "ARE YOU RUNNING?" => {
                                    todo!()
                                }
                                "START CAPTURE" => {
                                    self.state = StdoutHandlerState::InitCaptureHost(StreamState::InProgress("".to_string()));
                                }
                                "STOP CAPTURE" => {
                                    if let Some(capture) = &self.capturing {
                                        Self::write_prodlog_message(&mut self.stdout, &format!("Stopping capture of {} on {}:{}",
                                                                            capture.cmd,
                                                                            capture.host,
                                                                            capture.cwd))?;
                                        Self::stop_capturing(capture);
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
                            Self::write_prodlog_message(&mut self.stdout, &format!("Starting capture of {} on {}:{}", cmd, host, cwd))?;
                            self.capturing = Some(Self::start_capturing(host, cwd, &cmd));
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
    let _forward_stdout = tokio::spawn(async move {
        let mut buffer = [0; 1024];
        let mut stream_handler = StdoutHandler::new(raw_stdout);
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
    let result = match (unsafe { nix::pty::forkpty(None, None) }).unwrap() {
        ForkptyResult::Child => run_child(),
        ForkptyResult::Parent { child, master } => { run_parent(child, master).await }
    };
    if let Err(e) = result {
        eprintln!("PRODLOG EXITING WITH ERROR: {}", e);
        std::process::exit(1);
    } else {
        println!("PRODLOG EXITING");
        std::process::exit(0);
    }
}
