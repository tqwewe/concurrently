use crate::util::setup;

#[macro_use]
mod macros;
mod util;

#[test]
fn it_adds_two() {
    let (_, mut cmd) = setup("foo");
    let out = cmd.arg("sleep 1; echo first").arg("sleep 2; echo second").stdout();
    eqnice!("work in progress\n", out);
}