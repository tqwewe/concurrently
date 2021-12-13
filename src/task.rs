use std::process::{ExitStatus, Stdio};

use colored::{Color, Colorize};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::Mode;

#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub color: Color,
    pub tag_padding: usize,
    pub retries: usize,
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

    pub async fn prepare(&self, mode: Mode) -> io::Result<ExitStatus> {
        let mut cmd = Command::new("cargo");

        cmd.arg("build")
            .arg("--package")
            .arg(&self.name)
            .arg("--quiet");
        if mode == Mode::Release {
            cmd.arg("--release");
        }

        let tag = self.tag();
        let status = exec(cmd, &tag).await?;

        if status.success() {
            println!("{} {}", tag, "successfully built".bold().white());
        } else {
            println!("{} {}", tag, "failed to build".bold().red());
        }

        Ok(status)
    }

    pub async fn run(&self, mode: Mode) -> io::Result<ExitStatus> {
        let cmd = Command::new(format!("./target/{}/{}", mode.to_string(), self.name));

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
