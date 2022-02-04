use crate::util::setup;
use pretty_assertions::assert_str_eq;

#[macro_use]
mod util;

#[test]
fn it_runs_basic_commands() {
    let (_, mut cmd) = setup("foo");
    let out = cmd
        .arg("sleep 0.5; echo World")
        .arg("sleep 0.1; echo Hello")
        .stdout();
    let expected = r#"--> Running tasks
 Task 1  Running sleep 0.5; echo World
 Task 2  Running sleep 0.1; echo Hello
 Task 2  Hello
 Task 2  process exited with status code exit status: 0
 Task 1  World
 Task 1  process exited with status code exit status: 0
"#;
    assert_str_eq!(expected, out);
}
