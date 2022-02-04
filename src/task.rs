use std::{
    process::{ExitStatus, Stdio},
    time::Duration,
};

use crate::TaskNamer;
use colored::Colorize;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    process::Command,
    time,
};

#[derive(Clone, Debug)]
pub struct Task {
    pub name: String,
    pub command: String,
    pub tag_padding: usize,
    /// Number of retries already executed
    pub retries: usize,
    /// Maximum number of retries we will attempt
    pub max_retries: usize,
    pub delay: Option<Duration>,
    pub last_exit_status: Option<ExitStatus>,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            name: "default".to_string(),
            command: "true".to_string(),
            tag_padding: 0,
            retries: 0,
            max_retries: 3,
            delay: None,
            last_exit_status: None,
        }
    }
}

impl Task {
    pub fn from_cli(command: String, counter: &TaskNamer) -> Self {
        let name = counter.next();
        Task {
            name,
            command,
            ..Default::default()
        }
    }

    fn tag(&self) -> String {
        let mut tag = " ".repeat(self.tag_padding);
        tag.push_str(self.name.as_str());
        tag
    }

    pub fn should_retry(&self) -> bool {
        false
    }

    pub fn in_error_state(&self) -> bool {
        self.last_exit_status.map(|s| !s.success()).unwrap_or(false)
    }

    pub async fn spawn(mut self) -> Result<Task, io::Error> {
        if let Some(delay) = self.delay {
            time::sleep(delay).await;
        }

        let status = self.run().await?;
        self.last_exit_status = Some(status);
        Result::<_, io::Error>::Ok(self)
    }

    pub async fn run(&self) -> io::Result<ExitStatus> {
        let cmd = {
            println!("{} {}", self.tag(), format!("Running {}", self.command).white());
            let mut cmd = Command::new("sh");
            cmd.kill_on_drop(true);
            cmd.arg("-c");
            cmd.arg(self.command.replace('\n', " "));
            cmd
        };

        let status = exec(cmd, &self.tag()).await?;

        let mut status_message = format!("process exited with status code {}", status).bold();
        if status.success() {
            status_message = status_message.white();
        } else {
            status_message = status_message.red();
        }

        println!("{} {}", self.tag(), status_message);

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
                println!("{} {}", tag_cloned, line);
            }
        }
    });

    let status = child
        .wait()
        .await
        .expect("child process encountered an error");

    Ok(status)
}
