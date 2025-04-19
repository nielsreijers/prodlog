use nix::sys::wait::waitpid;
use std::fs::File;
use std::io::{ Read, Write };
use std::os::fd::{AsRawFd, RawFd};
use termion::raw::IntoRawMode;
use termion::input::TermReadEventsAndRaw;
use nix::pty::{ForkptyResult, Winsize};
use nix::ioctl_write_ptr_bad;
use termion::terminal_size;
use tokio::sync::mpsc;
use tokio::signal::unix::{signal, SignalKind};
use nix::unistd::execvp;
use std::ffi::CString;

fn run_child() -> Result<(), std::io::Error> {
    let cmd = CString::new("/bin/bash").expect("CString::new failed");
    let args = [
        CString::new("bash").expect("CString::new failed"),
    ];

    // Replace the current process with bash
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

async fn run_parent(child: nix::unistd::Pid, master: std::os::fd::OwnedFd) -> Result<(), std::io::Error> {
    // Set terminal to raw mode
    let mut raw_stdout = std::io::stdout().into_raw_mode()?;

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
    let stdin_reader_thread = tokio::task::spawn_blocking(move || {
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
        loop {
            let n = raw_master_read.read(&mut buffer);
            if let Ok(n) = n {
                if n == 0 {
                    break; // EOF reached
                }
                raw_stdout.write(&buffer[..n]).unwrap();
                raw_stdout.flush().unwrap();
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
        ForkptyResult::Parent { child, master } => {
            run_parent(child, master).await
        }
    };
    if let Err(e) = result {
        eprintln!("PRODLOG EXITING WITH ERROR: {}", e);
        std::process::exit(1);
    } else {
        println!("PRODLOG EXITING");
        std::process::exit(0);
    }
}