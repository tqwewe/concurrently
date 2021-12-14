use std::{
    process::{ExitStatus, Stdio},
    time::Duration,
};

use colored::{Color, Colorize};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    process::Command,
};

#[derive(Clone)]
pub enum TaskType {
    CargoWorkspaceMember(String, bool),
    Command(Vec<String>),
}

#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub task_type: TaskType,
    pub color: Color,
    pub tag_padding: usize,
    pub retries: usize,
    pub max_retries: usize,
    pub delay: Option<Duration>,
}

impl Task {
    fn tag(&self) -> String {
        let mut tag = " ".repeat(self.tag_padding);
        tag.push_str(
            &format!(" {} ", self.name.to_uppercase())
                .bold()
                .on_color(self.color)
                .truecolor(0, 0, 0)
                .to_string(),
        );
        tag.push_str("  ");
        tag
    }

    pub async fn prepare(&self) -> Option<io::Result<ExitStatus>> {
        let cmd = match &self.task_type {
            TaskType::CargoWorkspaceMember(name, release) => {
                let mut cmd = Command::new("cargo");

                cmd.arg("build").arg("--package").arg(name).arg("--quiet");

                if *release {
                    cmd.arg("--release");
                }

                cmd
            }
            TaskType::Command(_) => return None,
        };

        let tag = self.tag();
        let status = match exec(cmd, &tag).await {
            Ok(status) => status,
            Err(err) => return Some(Err(err)),
        };

        if status.success() {
            println!("{} {}", tag, "successfully built".bold().white());
        } else {
            println!("{} {}", tag, "failed to build".bold().red());
        }

        Some(Ok(status))
    }

    pub async fn run(&self) -> io::Result<ExitStatus> {
        let cmd = match &self.task_type {
            TaskType::CargoWorkspaceMember(name, release) => Command::new(format!(
                "./target/{}/{}",
                if *release { "release" } else { "debug" },
                name
            )),
            TaskType::Command(args) => {
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                // let mut cmd = Command::new(args.first().unwrap());
                cmd.arg(args.join(" "));
                // cmd.env(
                //     "DATABASE_URL",
                //     "postgres://auth:9c46082fb99e381521205b7f@127.0.0.1:6001/auth",
                // );
                // cmd.env("REDPANDA_HOST", "127.0.0.1:9092");
                cmd
            }
        };

        let tag = self.tag();
        let status = exec(cmd, &tag).await?;

        if status.success() {
            println!(
                "{} {}",
                tag,
                format!("process exited with status code {}", status)
                    .bold()
                    .white()
            );
        } else {
            println!(
                "{} {}",
                tag,
                format!("process exited with status code {}", status)
                    .bold()
                    .red()
            );
        }

        Ok(status)
    }
}

async fn exec(mut cmd: Command, tag: &str) -> io::Result<ExitStatus> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    let stderr = child
        .stderr
        .take()
        .expect("child did not have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let tag_cloned = tag.to_string();
    tokio::spawn(async move {
        while let Some(line) = stdout_reader.next_line().await.unwrap() {
            if !line.trim().is_empty() {
                println!("{} {}", tag_cloned, line);
            }
        }
    });

    let tag_cloned = tag.to_string();
    tokio::spawn(async move {
        while let Some(line) = stderr_reader.next_line().await.unwrap() {
            if !line.trim().is_empty() {
                println!("{} {}", tag_cloned, line.red());
            }
        }
    });

    let status = child
        .wait()
        .await
        .expect("child process encountered an error");

    Ok(status)
}
