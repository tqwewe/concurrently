use colored::Colorize;

use std::{cell::RefCell, time::Duration};

use colored::Color;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::time;

use crate::{
    log::{info, warn},
    task::Task,
};

use anyhow::{bail, Result};

pub mod config;
pub mod log;
pub mod task;

const COLORS: [Color; 9] = [
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

/// Utility for naming and colorizing the spawned tasks
pub struct TaskNamer {
    count: RefCell<usize>,
}

impl TaskNamer {
    pub fn new() -> Self {
        TaskNamer {
            count: RefCell::new(0),
        }
    }

    pub fn next(&self) -> String {
        let idx = *self.count.borrow() + 1;
        let name = format!("Task {}", idx);
        self.next_with_name(name)
    }

    pub fn next_with_name(&self, name: impl ToString) -> String {
        let mut count = self.count.borrow_mut();
        let name = format!(" {} ", name.to_string())
            .bold()
            .white()
            .on_color(COLORS[*count])
            .to_string();
        *count += 1;
        name
    }
}

pub async fn run_tasks(tasks: Vec<Task>, fail_on_error: bool) -> Result<()> {
    if tasks.is_empty() {
        info("nothing to run");
        return Ok(());
    }

    let mut workers = FuturesUnordered::new();
    for task in &tasks {
        let future = task.clone().spawn();
        workers.push(future);
    }

    while let Some(result) = workers.next().await {
        let mut task = result?;

        if task.in_error_state() && fail_on_error {
            bail!("'{:?}' failed. Aborting.", task.command)
        }

        if !task.should_retry() {
            continue;
        }

        let sleep_secs = (task.retries + 1) as u64;
        warn(format!(
            "task exited with non-success code, retrying again in {} seconds...",
            sleep_secs
        ));

        time::sleep(Duration::from_secs(sleep_secs)).await;
        task.retries += 1;
        workers.push(task.spawn());
    }

    Ok(())
}
