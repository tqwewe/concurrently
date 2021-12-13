use std::{fmt, process, time::Duration};

use cargo_toml::Manifest;
use clap::Parser;
use colored::Color;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use log::{error, info};
use tokio::{io, time};

use crate::{
    log::{warn, UnwrapResult},
    task::Task,
};

mod config;
mod log;
mod task;

const COLORS: [Color; 10] = [
    Color::Green,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Yellow,
    Color::BrightGreen,
    Color::BrightBlue,
    Color::BrightMagenta,
    Color::BrightCyan,
    Color::BrightYellow,
];

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Run all workspace members
    #[clap(long)]
    all: bool,
    /// Additional raw commands
    #[clap(global = true)]
    commands: Vec<String>,
    /// Run cargo workspace members in release mode
    #[clap(long)]
    release: bool,
    /// Run specific cargo workspace members (comma separated)
    #[clap(short, long)]
    members: Vec<String>,
    /// Location of workspace Cargo.toml
    #[clap(short, long, default_value = "./Cargo.toml")]
    workspace: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Debug,
    Release,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Debug => write!(f, "debug"),
            Mode::Release => write!(f, "release"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mode = if args.release {
        Mode::Release
    } else {
        Mode::Debug
    };

    let members: Vec<_> = if args.all {
        let members = Manifest::from_path(args.workspace)
            .map(|manifest| {
                manifest
                    .workspace
                    .map(|workspace| workspace.members)
                    .unwrap_warn("no members in workspace")
            })
            .unwrap_error("could not read workspace Cargo.toml file");
        if members.is_empty() {
            warn("no members in workspace");
            process::exit(0);
        }
        members
    } else if args.memb {
        args.members
            .into_iter()
            .flat_map(|member_string| {
                member_string
                    .split(',')
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .collect()
    };

    let longest_name = members.iter().fold(0, |acc, member| {
        if member.len() > acc {
            member.len()
        } else {
            acc
        }
    });

    let mut next_color: usize = 0;
    let tasks: Vec<_> = members
        .into_iter()
        .map(|name| {
            let color = COLORS[next_color];
            next_color += 1;
            if next_color > 10 {
                next_color = 0;
            }
            let tag_padding = longest_name - name.len();
            Task {
                name,
                color,
                tag_padding,
                retries: 0,
            }
        })
        .collect();

    if tasks.is_empty() {
        info("nothing to run");
    }

    info("preparing tasks...");
    let mut workers = FuturesUnordered::new();
    for task in &tasks {
        workers.push(async move { task.prepare(mode).await });
    }
    while let Some(result) = workers.next().await {
        if !result.map(|status| status.success()).unwrap_or(false) {
            error("task preparation failed");
            process::exit(1);
        }
    }

    info("running tasks...");
    let mut workers = FuturesUnordered::new();
    for task in &tasks {
        let task = task.clone();
        workers.push(Box::pin(
            async move {
                let status = task.run(mode).await?;
                Result::<_, io::Error>::Ok((status, task))
            }
            .boxed(),
        ));
    }
    while let Some(result) = workers.next().await {
        if let Ok((status, mut task)) = result {
            if task.retries > 3 && !status.success() {
                error("task exited with non-success code too many times, exiting.");
                return Ok(());
            }
            let sleep_secs = (task.retries + 1) as u64;
            warn(format!(
                "task exited with non-success code, retrying again in {} seconds...",
                sleep_secs
            ));
            time::sleep(Duration::from_secs(sleep_secs)).await;
            task.retries += 1;
            workers.push(Box::pin(
                async move {
                    let status = task.run(mode).await?;
                    Result::<_, io::Error>::Ok((status, task))
                }
                .boxed(),
            ));
        }
    }

    Ok(())
}
