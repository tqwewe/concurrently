# Concurrently

Run multiple processes concurrently.

_Written in Rust with ☕_

## Install with Cargo

```bash
$ cargo install concurrently
```

## Getting Started

Create a `tasks.toml` file in the root of your project:

```toml
[tasks.api]
cargo_workspace_member = true # Run the api cargo package
retries = 3 # Retry 3 times before exiting
delay = 1000 # Wait 1 second before running

[tasks.domain]
cargo_workspace_member = true # Run the api cargo package
release = true # Run in --release mode

[tasks.relay]
# Run a custom command
command = [
  "docker",
  "run",
  "--name=auth-outbox-relay",
  "--net=host",
  "--init",
  "--rm",
  "acidic9/outbox-relay:latest",
  "./outbox-relay -d $DATABASE_URL -r $REDPANDA_HOST",
]
retries = 3 # Retry 3 times before exiting
delay = 1000 # Wait 1 second before running
```

Now you can simply run concurrently:

```bash
$ cargo concurrently
```

<p align="center">
  <img src="https://raw.githubusercontent.com/Acidic9/concurrently/main/terminal-screenshot.png">
</p>

## Good to Know 

- watch -n1 -d echo 'Hello from Demo 1 - $(date)' - nice command to generate endless process, problem: does not write to stdout