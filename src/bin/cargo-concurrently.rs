extern crate concurrently;

use std::{process, time::Duration};

use clap::Parser;
use colored::Color;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use tokio::{fs, io, time};

use concurrently::task::TaskType;
use concurrently::log::{error, info};
use concurrently::config::Config;
use concurrently::{log::warn, task::Task};

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
    /// Tasks to run
    #[clap(global = true)]
    tasks: Vec<String>,

    #[clap(short, long)]
    file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("{:?}", args);
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



    let tasks_file = fs::read_to_string("./tasks.toml").await?;
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

    let tasks: Vec<_> = config
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
    let longest_name = tasks.iter().fold(
        0,
        |acc, (name, _)| {
            if name.len() > acc {
                name.len()
            } else {
                acc
            }
        },
    );
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
            let task_type = if let Some(cmd) = opts.command {
                if cmd.is_empty() {
                    warn(format!("task '{}' has an empty command, ignoring.", name));
                    return None;
                } else {
                    TaskType::Command(cmd)
                }
            } else if opts.cargo_workspace_member {
                TaskType::CargoWorkspaceMember(name.clone(), opts.release)
            } else {
                return None;
            };
            Some(Task {
                name,
                task_type,
                color,
                prepare: opts.prepare,
                tag_padding,
                retries: 0,
                max_retries: opts.retries,
                delay: opts.delay.map(Duration::from_millis),
            })
        })
        .collect();

    if tasks.is_empty() {
        info("nothing to run");
    }

    info("preparing tasks...");
    let mut workers = FuturesUnordered::new();
    for task in &tasks {
        workers.push(async move { task.prepare().await });
    }
    while let Some(result) = workers.next().await {
        if let Some(result) = result {
            if !result.map(|status| status.success()).unwrap_or(false) {
                error("task preparation failed");
                process::exit(1);
            }
        }
    }

    info("running tasks...");
    let mut workers = FuturesUnordered::new();
    for task in &tasks {
        let task = task.clone();
        workers.push(Box::pin(
            async move {
                if let Some(delay) = task.delay {
                    time::sleep(delay).await;
                }
                let status = task.run().await?;
                Result::<_, io::Error>::Ok((status, task))
            }
            .boxed(),
        ));
    }
    while let Some(result) = workers.next().await {
        if let Ok((status, mut task)) = result {
            if task.retries > 3 && !status.success() {
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

    Ok(())
}
