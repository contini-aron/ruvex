use crate::errors::CheckError;
use prettytable::{color, Attr, Cell, Row, Table};
use ruvex_config::Config;
use ruvex_utils::ConventionalCommit;
use std::process::Command;

//apply \n each n chars
fn return_nch(to_parse: &str, n_char: usize) -> String {
    to_parse
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i != 0 && i % n_char == 0 {
                Some('\n')
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect::<String>()
}

pub fn check(
    name: Option<String>,
    diff: Option<String>,
    return_n: Option<usize>,
    config: &Config,
) -> Result<(), CheckError> {
    let mut cmd = Command::new("git");
    cmd.arg("log");
    cmd.arg("--no-decorate");
    cmd.arg("--format=\"%h%n%B\"");
    if let Some(name) = name {
        if let Some(diff) = diff {
            cmd.arg([diff, "..".to_string(), name].concat());
        } else {
            cmd.arg(name);
        }
    }
    let result = String::from_utf8(cmd.output().unwrap().stdout).unwrap();
    //println!("{:#?}", &result);
    let rows = result.split("\"\n");
    //println!("{:#?}", &rows);

    let mut commits: Vec<ConventionalCommit> = Vec::new();

    //add title to error table
    let mut err_commits: Table = Table::new();
    err_commits.add_row(Row::new(vec![
        Cell::new("Short SHA")
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
        Cell::new("Commit Message")
            .with_style(Attr::ForegroundColor(color::RED))
            .with_style(Attr::Italic(true))
            .with_hspan(2),
        Cell::new("Reason")
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
    ]));
    for row in rows {
        // break if last line
        if row.is_empty() {
            break;
        }

        let (short_sha, commit_msg) = row.split_once('\n').unwrap();
        let trim_sha = short_sha.replace('"', "");
        match ConventionalCommit::new(commit_msg, config, trim_sha.clone()) {
            Ok(cc) => commits.push(cc),
            Err(err_msg) => {
                err_commits.add_row(Row::new(vec![
                    Cell::new(&trim_sha)
                        .with_style(Attr::Bold)
                        .with_style(Attr::ForegroundColor(color::RED)),
                    Cell::new(&return_nch(commit_msg, return_n.unwrap_or(40)))
                        .with_style(Attr::ForegroundColor(color::RED))
                        .with_style(Attr::Italic(true))
                        .with_hspan(2),
                    Cell::new(&err_msg.to_string())
                        .with_style(Attr::Bold)
                        .with_style(Attr::ForegroundColor(color::RED)),
                ]));
            }
        }
    }
    if err_commits.len() > 1 {
        err_commits.printstd();
        Err(CheckError::CCNotCompliant(format!(
            "found {} bad commits out of {}",
            err_commits.len() - 1,
            commits.len() + err_commits.len()
        )))
    } else {
        Ok(())
    }
}
