# Concurrently

Run multiple processes concurrently, with support for cargo.

_Written in Rust with ☕_

## Install with Cargo

```bash
$ cargo install concurrently
```

## Getting Started

Create a `tasks.toml` file in the root of your project:

```toml
[tasks.client]
workspace = true  # Run the client cargo workspace member
retries = 3       # Retry 3 times before exiting
delay = "1s"      # Wait 1 second before running

[tasks.server]
workspace = true  # Run the server cargo workspace member
release = true    # Run in --release mode

[tasks.db]
command = [
  "docker",
  "run",
  "postgres",
]
```

Now you can simply run concurrently:

```bash
$ cargo concurrently
```

## Config

**Common**

These configs are optional, and can be used with all tasks.

| Config  | Type   |                                                                                  |
|---------|--------|----------------------------------------------------------------------------------|
| prepare | String | Runs a command before starting the task.                                         |
| delay   | String | Waits before starting the task. This can be in the format of "1s", "100ms", etc. |
| retries | Number | Retries this task before exiting all other tasks.                                |

**Shell Task**

Shell task runs a shell command.

| Config  | Type            |                               |
|---------|-----------------|-------------------------------|
| command | String or Array | Runs the command as the task. |

**Cargo Task**

Cargo tasks are built using cargo with `cargo build -p <name>` where `name` is the name of the task.

`cargo` must be set to `true` for a task to be a cargo task.

| Config   | Type     |                                                                                        |
|----------|----------|----------------------------------------------------------------------------------------|
| cargo    | Bool     | If set to true, treats this task as a cargo crate. The crate will be built on startup. |
| release  | Bool     | Builds for release.                                                                    |
| features | [String] | Array of feature flags.                                                                |
