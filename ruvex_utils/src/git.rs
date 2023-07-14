use anyhow::Result;
use colored::Colorize;
use std::process::Command;
use std::process::Output;

fn check_git_error(stderr: Vec<u8>) -> Result<()> {
    if !stderr.is_empty() {
        return Err(anyhow::Error::msg(
            format!("git command error:\n{}", String::from_utf8(stderr).unwrap()).red(),
        ));
    }
    Ok(())
}

fn generic_git_cmd(args: &[&str], subcmd: &str) -> Result<Output> {
    let mut cmd = Command::new("git");
    cmd.arg(subcmd);
    for arg in args.iter() {
        cmd.arg(arg);
    }
    let retval = cmd.output()?;
    check_git_error(retval.stderr.clone())?;
    Ok(retval)
}

pub fn log(args: &[&str]) -> Result<Output> {
    generic_git_cmd(args, "log")
}

pub fn tag(args: &[&str]) -> Result<Output> {
    generic_git_cmd(args, "tag")
}
