use std::collections::HashMap;

use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub tasks: HashMap<String, TaskOptions>,
    pub env: Option<String>,
}

#[derive(Default, Deserialize, Debug)]
#[serde(default)]
pub struct TaskOptions {
    pub env_file: Option<String>,
    pub before_command: Option<String>,
    pub command: String,
    pub delay: Option<u64>,
    pub retries: usize,
}
