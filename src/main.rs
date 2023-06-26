use std::{process, sync::Arc, time::Duration};

use anyhow::Context;
use clap::Parser;
use colored::Color;
use config::Config;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{error, info};
use tokio::{fs, io, time};

use crate::{log::warn, task::Task};

mod config;
mod log;
mod task;

const COLORS: [Color; 10] = [
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
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
    /// Tasks to run
    #[clap(global = true)]
    tasks: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let selected_tasks: Vec<_> = args
        .tasks
        .into_iter()
        .flat_map(|member_string| {
            member_string
                .split(',')
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .collect();

    let tasks_file = fs::read_to_string("./tasks.toml")
        .await
        .context("no tasks.toml found")?;
    let config: Config = toml::from_str(&tasks_file)?;

    if let Some(env) = config.env {
        match dotenv::from_filename(&env) {
            Ok(_) => {
                info(format!("loaded env file {}", env));
            }
            Err(_) => {
                warn(format!("failed to load env file {}", env));
            }
        }
    }

    let mut tasks: Vec<_> = config
        .tasks
        .into_iter()
        .filter(|task| {
            if selected_tasks.is_empty() {
                true
            } else {
                selected_tasks
                    .iter()
                    .any(|selected_task| *selected_task == task.0)
            }
        })
        .collect();
    tasks.sort_by(|(a, _), (b, _)| a.cmp(b));
    let longest_name = tasks.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    let mut next_color: usize = 0;

    let tasks: Vec<_> = tasks
        .into_iter()
        .filter_map(|(name, opts)| {
            let color = COLORS[next_color];
            next_color += 1;
            if next_color > 10 {
                next_color = 0;
            }
            let tag_padding = longest_name - name.len();
            Some(Task::from_options(name, color, tag_padding, opts))
        })
        .collect();

    if tasks.is_empty() {
        info("nothing to run");
        return Ok(());
    }

    // info("preparing tasks...");

    let m = MultiProgress::new();
    m.set_move_cursor(true);
    let sty =
        ProgressStyle::with_template("{prefix} {spinner:.bold/white} {wide_msg:.bold/white/!}")
            .unwrap();

    let spinners = Arc::new(
        tasks
            .iter()
            .map(|task| {
                let spinner = m
                    .add(ProgressBar::new_spinner())
                    .with_prefix(task.tag.clone());
                spinner.set_style(sty.clone());
                spinner
            })
            .collect::<Vec<_>>(),
    );

    let ticker = {
        let spinners = Arc::clone(&spinners);
        tokio::spawn(async move {
            loop {
                for spinner in spinners.iter() {
                    spinner.tick();
                }
                time::sleep(time::Duration::from_millis(100)).await;
            }
        })
    };

    let mut workers = FuturesUnordered::new();
    for (i, task) in tasks.iter().enumerate() {
        let spinner = spinners[i].clone();
        workers.push(async move {
            let result = task.prepare(spinner.clone()).await;
            match &result {
                Some(Ok(status)) => {
                    if status.success() {
                        spinner.finish_with_message("done");
                    } else {
                        match status.code() {
                            Some(code) => {
                                spinner
                                    .finish_with_message(format!("failed with exit code {code}",));
                            }
                            None => {
                                spinner.finish_with_message("failed");
                            }
                        }
                    }
                }
                Some(Err(err)) => {
                    spinner.println(err.to_string());
                    spinner.finish();
                }
                None => {
                    spinner.finish_and_clear();
                }
            }
            result
        });
    }
    while let Some(result) = workers.next().await {
        if let Some(result) = result {
            if !result.map(|status| status.success()).unwrap_or(false) {
                error("task preparation failed");
                process::exit(1);
            }
        }
    }

    ticker.abort();
    let _ = m.clear();

    // info("running tasks...");
    let mut workers = FuturesUnordered::new();
    for task in &tasks {
        let task = task.clone();
        workers.push(Box::pin(
            async move {
                let status = task.run().await?;
                Result::<_, io::Error>::Ok((status, task))
            }
            .boxed(),
        ));
    }
    while let Some(result) = workers.next().await {
        if let Ok((status, mut task)) = result {
            if !status.success() {
                if task.retries > task.max_retries {
                    error(format!(
                        "task {} exited with non-success code too many times, exiting.",
                        task.name
                    ));
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
                        let status = task.run().await?;
                        Result::<_, io::Error>::Ok((status, task))
                    }
                    .boxed(),
                ));
            }
        }
    }

    Ok(())
}
