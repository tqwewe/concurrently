#![allow(
    clippy::empty_enum,
    clippy::let_underscore_untyped,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args
)]

use nix::fcntl::{self, FcntlArg, FdFlag};
use nix::pty::{self, ForkptyResult, Winsize};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, ForkResult, Pid};
use nix::Result;
use std::ffi::CString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::RawFd;
use std::process;

enum Exec {}

pub fn run() {
    match try_main() {
        Ok(exec) => match exec {},
        Err(err) => {
            let _ = writeln!(io::stderr(), "faketty: {}", err);
            process::exit(1);
        }
    }
}

fn try_main() -> Result<Exec> {
    let args = args();
    let stdin = dup(0)?;
    let stderr = dup(2)?;
    let pty1 = unsafe { forkpty() }?;
    if let ForkResult::Parent { child } = pty1.fork_result {
        copyfd(pty1.master, 1);
        copyexit(child);
    }
    let stdout = dup(1)?;
    let pty2 = unsafe { forkpty() }?;
    if let ForkResult::Parent { child } = pty2.fork_result {
        copyfd(pty2.master, stderr);
        copyexit(child);
    }
    unistd::dup2(stdin, 0)?;
    unistd::dup2(stdout, 1)?;
    exec(args)
}

fn args() -> Vec<CString> {
    std::env::args_os()
        .skip(2)
        .map(|os_string| CString::new(os_string.as_bytes()).unwrap())
        .collect()
}

fn dup(fd: RawFd) -> Result<RawFd> {
    let new = unistd::dup(fd)?;
    fcntl::fcntl(new, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?;
    Ok(new)
}

unsafe fn forkpty() -> Result<ForkptyResult> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let termios = None;
    pty::forkpty(&winsize, termios)
}

fn exec(args: Vec<CString>) -> Result<Exec> {
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    unistd::execvp(args[0], &args)?;
    unreachable!();
}

fn copyfd(read: RawFd, write: RawFd) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(read, &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let _ = write_all(write, &buf[..n]);
            }
        }
    }
}

fn write_all(fd: RawFd, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        let n = unistd::write(fd, buf)?;
        buf = &buf[n..];
    }
    Ok(())
}

fn copyexit(child: Pid) -> ! {
    let flag = None;
    process::exit(match wait::waitpid(child, flag) {
        Ok(WaitStatus::Exited(_pid, code)) => code,
        _ => 0,
    });
}
