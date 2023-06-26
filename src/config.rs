use std::{collections::HashMap, fmt, time::Duration};

use serde::{de, Deserialize};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub tasks: HashMap<String, TaskOptions>,
    pub env: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TaskOptions {
    #[serde(flatten)]
    pub task_options: TaskTypeOptions,
    #[serde(default)]
    pub prepare: Option<String>,
    // pub env_file: Option<String>, // Allow hard-coded env, or env file
    #[serde(default, with = "humantime_serde")]
    pub delay: Option<Duration>,
    #[serde(default)]
    pub retries: usize,
}

#[derive(Clone, Debug)]
pub enum TaskTypeOptions {
    Shell(ShellTaskOptions),
    Cargo(CargoTaskOptions),
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CargoTaskOptions {
    pub release: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShellTaskOptions {
    pub command: Command,
}

#[derive(Clone, Debug)]
pub enum Command {
    String(String),
    Args(Vec<String>),
}

impl Command {
    pub fn is_empty(&self) -> bool {
        match self {
            Command::String(s) => s.trim().is_empty(),
            Command::Args(args) => args.join(" ").trim().is_empty(),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::String(s) => write!(f, "{}", s.trim().replace('\n', " ")),
            Command::Args(args) => write!(f, "{}", args.join(" ").trim().replace('\n', " ")),
        }
    }
}

impl<'de> Deserialize<'de> for TaskTypeOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut table = toml::Table::deserialize(deserializer)?;
        match table.remove("cargo") {
            Some(toml::Value::Boolean(true)) => {
                let opts = CargoTaskOptions::deserialize(table)
                    .map_err(|err| serde::de::Error::custom(err.message()))?;
                Ok(TaskTypeOptions::Cargo(opts))
            }
            _ => {
                let opts = ShellTaskOptions::deserialize(table)
                    .map_err(|err| serde::de::Error::custom(err.message()))?;
                Ok(TaskTypeOptions::Shell(opts))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::String(s) => Ok(Command::String(s)),
            toml::Value::Array(a) => a
                .into_iter()
                .map(|v| match v {
                    toml::Value::String(s) => Ok(s),
                    toml::Value::Integer(i) => Err(de::Error::invalid_type(
                        de::Unexpected::Signed(i),
                        &"string or array",
                    )),
                    toml::Value::Float(f) => Err(de::Error::invalid_type(
                        de::Unexpected::Float(f),
                        &"string or array",
                    )),
                    toml::Value::Boolean(b) => Err(de::Error::invalid_type(
                        de::Unexpected::Bool(b),
                        &"string or array",
                    )),
                    toml::Value::Datetime(_) => Err(de::Error::invalid_type(
                        de::Unexpected::Other("datetime"),
                        &"string or array",
                    )),
                    toml::Value::Array(_) => Err(de::Error::invalid_type(
                        de::Unexpected::Other("array"),
                        &"string or array",
                    )),
                    toml::Value::Table(_) => Err(de::Error::invalid_type(
                        de::Unexpected::Other("table"),
                        &"string or array",
                    )),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Command::Args),
            toml::Value::Integer(i) => Err(de::Error::invalid_type(
                de::Unexpected::Signed(i),
                &"string or array",
            )),
            toml::Value::Float(f) => Err(de::Error::invalid_type(
                de::Unexpected::Float(f),
                &"string or array",
            )),
            toml::Value::Boolean(b) => Err(de::Error::invalid_type(
                de::Unexpected::Bool(b),
                &"string or array",
            )),
            toml::Value::Datetime(_) => Err(de::Error::invalid_type(
                de::Unexpected::Other("datetime"),
                &"string or array",
            )),
            toml::Value::Table(_) => Err(de::Error::invalid_type(
                de::Unexpected::Other("table"),
                &"string or array",
            )),
        }
    }
}
