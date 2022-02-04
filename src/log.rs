use std::{fmt, process};

use colored::Colorize;

pub(crate) trait UnwrapResult<T> {
    fn unwrap_error(self, msg: impl fmt::Display) -> T;
    fn unwrap_warn(self, msg: impl fmt::Display) -> T;
}

impl<T, E> UnwrapResult<T> for Result<T, E> {
    fn unwrap_error(self, msg: impl fmt::Display) -> T {
        match self {
            Ok(val) => val,
            Err(_) => {
                error(msg);
                process::exit(1)
            }
        }
    }

    fn unwrap_warn(self, msg: impl fmt::Display) -> T {
        match self {
            Ok(val) => val,
            Err(_) => {
                warn(msg);
                process::exit(0)
            }
        }
    }
}

impl<T> UnwrapResult<T> for Option<T> {
    fn unwrap_error(self, msg: impl fmt::Display) -> T {
        match self {
            Some(val) => val,
            None => {
                error(msg);
                process::exit(1)
            }
        }
    }

    fn unwrap_warn(self, msg: impl fmt::Display) -> T {
        match self {
            Some(val) => val,
            None => {
                warn(msg);
                process::exit(0)
            }
        }
    }
}

pub fn info(msg: impl fmt::Display) {
    let msg = format!("--> {}", msg).white();
    println!("{}", msg);
}

pub fn error(msg: impl fmt::Display) {
    let tag = "[error] ".bold().red();
    println!("{} {}", tag, msg);
}

pub fn warn(msg: impl fmt::Display) {
    let tag = "[warn] ".bold().yellow();
    println!("{} {}", tag, msg);
}
