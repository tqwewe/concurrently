use std::collections::HashMap;

use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub tasks: HashMap<String, TaskOptions>,
    pub env: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct TaskOptions {
    pub env_file: Option<String>,
    pub cargo_workspace_member: bool,
    pub prepare: Option<String>,
    pub delay: Option<u64>,
    pub retries: usize,
    pub release: bool,
}
