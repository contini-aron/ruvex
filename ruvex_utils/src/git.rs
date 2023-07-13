use anyhow::Result;
use std::process::Command;
use std::process::Output;

pub fn log(args: &Vec<String>) -> Result<Output> {
    let mut cmd = Command::new("git");
    cmd.arg("log");
    for arg in args.iter() {
        cmd.arg(arg);
    }
    Ok(cmd.output()?)
}
