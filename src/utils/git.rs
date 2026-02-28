use anyhow::Result;
use colored::Colorize;
use std::process::Command;
use std::process::Output;
use git2::{Error, Oid, Repository, Revwalk, Sort};

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

pub fn open_repo(repo_path: &str) -> Repository {
    match Repository::open(repo_path) {
        std::result::Result::Ok(repo) => return repo,
        Err(e) => panic!("failed to open: {}", e),
    };
}
pub fn raw_log(repo_path: &str) -> Result<(), Error>{
    let repo = open_repo(repo_path);
    // let commit = match repo.find_commit_by_prefix("e047530"){
    //     std::result::Result::Ok(commit) => commit,
    //
    //     Err(e) => panic!("failed to open: {}", e),
    //
    // };
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;
    
    let revparse = repo.revparse("HEAD~4..")?;
    println!("from {:#?}", revparse.from().unwrap().id()); 
    println!("to {:#?}", revparse.to().unwrap().id());
    let from = revparse.from().unwrap().id();
    let to = revparse.to().unwrap().id();
    revwalk.hide(from)?;
    revwalk.push(to)?;
    
    for id in revwalk {
        let commit = repo.find_commit(id?)?;
        // println!("{:#?}", commit.message());
        println!("{}", commit.summary().unwrap());
    }
    Ok(())
}


/////////////////////////////////////////////////////////////
