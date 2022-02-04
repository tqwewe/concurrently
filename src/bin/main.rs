extern crate concurrently;

use anyhow::Result;
use clap::Parser;
use tokio::fs;
use concurrently::{log::info, run_tasks, task::Task, TaskNamer};
use concurrently::config::Config;
use concurrently::log::warn;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(global = true)]
    params: Vec<String>,

    #[clap(short, long)]
    before: Vec<String>,

    #[clap(short, long)]
    file: Option<String>,
}

fn expand(param: impl AsRef<str>) -> String {
    let parts: Vec<&str> = param.as_ref().splitn(2, ":").collect();
    match parts.as_slice() {
        ["cargo", cmd] => format!("cargo run --color always --package {}", cmd),
        _ => param.as_ref().to_string(),
    }
}

fn read_tasks_from_file(contents: String, before_tasks: &mut Vec<Task>, tasks: &mut Vec<Task>) -> Result<()> {
    let namer = TaskNamer::new();
    let config: Config = toml::from_str(&contents)?;
    for (task_name, options) in config.tasks.into_iter() {
        tasks.push(Task {
            name: namer.next_with_name(&task_name),
            command: expand(options.command),
            max_retries: 3,
            delay: None,
            ..Task::default()
        });

        if let Some(cmd) = options.before_command {
            let name = namer.next_with_name(format!("(before) {}", &task_name));
            before_tasks.push(Task {
                name,
                command: cmd,
                max_retries: 3,
                delay: None,
                ..Task::default()
            });
        }
    }
    Ok(())
}

async fn maybe_read_tasks_from_file(file: String, before_tasks: &mut Vec<Task>, tasks: &mut Vec<Task>) -> Result<()> {
    match fs::read_to_string(&file).await {
        Err(e) => {
            warn(format!("Unable to read file '{}': {:?}", file, e));
            Ok(())
        }
        Ok(contents) => {
            read_tasks_from_file(contents, before_tasks, tasks)
        }

    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    let namer = TaskNamer::new();
    let mut tasks = vec![];
    let mut before_tasks = vec![];

    for param in &args.before {
        let task = Task::from_cli(expand(&param), &namer);
        before_tasks.push(task);
    }

    for param in &args.params {
        let task = Task::from_cli(expand(&param), &namer);
        tasks.push(task);
    }

    // No tasks passed via command line, let's look in the file
    if before_tasks.is_empty() && tasks.is_empty() {
        let file = args.file.unwrap_or("./concurrently.toml".to_string());
        maybe_read_tasks_from_file(file, &mut before_tasks, &mut tasks).await?;
    }

    if !before_tasks.is_empty() {
        info("Running 'before' tasks");
        run_tasks(before_tasks, false).await?;
    }

    info("Running tasks");
    run_tasks(tasks, false).await
}
