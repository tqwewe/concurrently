use std::{
    env,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use colored::{Color, Colorize};
use indicatif::ProgressBar;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    process::Command,
    time,
};

use crate::{
    config::{self, CargoTaskOptions, ShellTaskOptions, TaskOptions, TaskTypeOptions},
    log::warn,
};

#[derive(Clone)]
pub struct Task {
    pub name: String,
    prepare: Option<config::Command>,
    pub retries: usize,
    pub max_retries: usize,
    delay: Option<Duration>,
    pub tag: String,
    opts: TaskTypeOptions,
    current_exe: PathBuf,
}

impl Task {
    pub fn from_options(name: String, color: Color, tag_padding: usize, opts: TaskOptions) -> Self {
        let mut tag = name.bold().color(color).to_string();
        tag.push_str(&" ".repeat(tag_padding + 2));
        tag.push_str(&"|".bold().color(color).to_string());
        tag.push_str(" ");

        if let TaskTypeOptions::Shell(ShellTaskOptions { command }) = &opts.task_options {
            if command.is_empty() {
                warn(format!("task {name} has no command configured"));
            }
        }

        let current_exe =
            env::current_exe().expect("could not get path to currently running executable");

        Task {
            name,
            prepare: opts.prepare,
            retries: 0,
            max_retries: opts.retries,
            delay: opts.delay,
            tag,
            opts: opts.task_options,
            current_exe,
        }
    }

    pub async fn prepare(&self, pb: ProgressBar) -> Option<io::Result<ExitStatus>> {
        let result = match &self.opts {
            TaskTypeOptions::Shell(_) => None,
            TaskTypeOptions::Cargo(CargoTaskOptions { release }) => {
                // Build the project
                let mut cmd = Command::new(&self.current_exe);
                cmd.arg("--fake-tty")
                    .arg("cargo")
                    .arg("build")
                    .arg("-p")
                    .arg(&self.name)
                    .arg("--color=always");
                if *release {
                    cmd.arg("--release");
                }
                cmd.envs(env::vars());

                let status = match exec(cmd, &self.tag, Some(pb.clone())).await {
                    Ok(status) => status,
                    Err(err) => return Some(Err(err)),
                };

                if status.success() {
                    // println!("{} {}", self.tag, "successfully built".bold().white());
                } else {
                    // println!("{} {}", self.tag, "failed to build".bold().red());
                    return Some(Ok(status));
                }
                Some(Ok(status))
            }
        };

        if let Some(prepare) = &self.prepare {
            let mut cmd = Command::new(&self.current_exe);
            cmd.arg("--fake-tty");
            cmd.arg("sh");
            cmd.arg("-c");
            cmd.arg(prepare.to_string());
            cmd.envs(env::vars());

            let status = match exec(cmd, &self.tag, Some(pb)).await {
                Ok(status) => status,
                Err(err) => return Some(Err(err)),
            };

            if status.success() {
                // println!(
                //     "{} {}",
                //     self.tag,
                //     format!("process exited with status code {status}")
                //         .bold()
                //         .white()
                // );
            } else {
                println!(
                    "{} {}",
                    self.tag,
                    format!("process exited with status code {status}")
                        .bold()
                        .red()
                );
            }

            Some(Ok(status))
        } else {
            result
        }
    }

    pub async fn run(&self) -> io::Result<ExitStatus> {
        if let Some(delay) = self.delay {
            println!(
                "{} {}",
                self.tag,
                format!("waiting {:.2}s", delay.as_secs_f32())
                    .bold()
                    .white()
            );
            time::sleep(delay).await;
        }

        let mut cmd = match &self.opts {
            TaskTypeOptions::Shell(ShellTaskOptions { command }) => {
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                cmd.arg(command.to_string());
                cmd
            }
            TaskTypeOptions::Cargo(CargoTaskOptions { release }) => Command::new(format!(
                "./target/{}/{}",
                if *release { "release" } else { "debug" },
                &self.name
            )),
        };
        cmd.envs(env::vars());

        let status = exec(cmd, &self.tag, None).await?;

        if status.success() {
            println!(
                "{} {}",
                self.tag,
                format!("process exited with status code {status}")
                    .bold()
                    .white()
            );
        } else {
            println!(
                "{} {}",
                self.tag,
                format!("process exited with status code {status}")
                    .bold()
                    .red()
            );
        }

        Ok(status)
    }
}

async fn exec(mut cmd: Command, tag: &str, pb: Option<ProgressBar>) -> io::Result<ExitStatus> {
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
    let mut stderr_reader = BufReader::with_capacity(1024, stderr).lines();

    let stdout_task = {
        let tag = tag.to_string();
        let pb = pb.clone();
        tokio::spawn(async move {
            while let Some(line) = stdout_reader.next_line().await.unwrap() {
                if !line.trim().is_empty() {
                    if let Some(pb) = &pb {
                        pb.set_message(line);
                    } else {
                        println!("{tag} {}", line);
                    }
                }
            }
        })
    };

    let stderr_task = {
        let tag = tag.to_string();
        let pb = pb.clone();
        tokio::spawn(async move {
            let mut last_ten_lines = Vec::with_capacity(20);
            while let Some(line) = stderr_reader.next_line().await.unwrap() {
                if !line.trim().is_empty() {
                    if let Some(pb) = &pb {
                        pb.set_message(line.clone());
                    } else {
                        println!("{tag} {}", line.red());
                    }
                    if last_ten_lines.len() >= 20 {
                        last_ten_lines.remove(0);
                    }
                    last_ten_lines.push(line);
                }
            }
            last_ten_lines
        })
    };

    let status = child
        .wait()
        .await
        .expect("child process encountered an error");

    let _ = stdout_task.await.unwrap();
    let last_ten_lines = stderr_task.await.unwrap();

    if !status.success() {
        if let Some(pb) = pb {
            pb.println(
                "showing last 20 lines of stderr..."
                    .bold()
                    .white()
                    .to_string(),
            );
            for line in last_ten_lines {
                pb.println(line);
            }
        }
    }

    Ok(status)
}
