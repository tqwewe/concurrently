use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub tasks: HashMap<String, TaskOptions>,
}

#[derive(Deserialize)]
pub struct TaskOptions {
    pub cargo_workspace_member: bool,
    pub command: Vec<String>,
    pub retries: usize,
}
