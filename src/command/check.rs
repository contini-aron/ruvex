use crate::config::Config;
use crate::utils::{git, ConventionalCommit};
use colored::Colorize;
use log::{debug, info};
use prettytable::{color, Attr, Cell, Row, Table};

/// Visual separator used to delimit the check output sections in the terminal.
const SEPARATOR: &str = "################################################";

/// Default character width before wrapping long text in table cells.
/// Used as a fallback when `return_n` is not provided.
const DEFAULT_WRAP_WIDTH: usize = 40;

/// Inserts a newline character every `n_char` characters to prevent long strings
/// from overflowing table cells in the terminal output.
///
/// # Example
/// let wrapped = wrap_text("abcdefgh", 3);
/// assert_eq!(wrapped, "abc\ndef\ngh");
///
fn wrap_text(text: &str, n_char: usize) -> String {
    text.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            // Insert a newline before every `n_char`-th character (except the first)
            let newline = (i != 0 && i % n_char == 0).then_some('\n');
            newline.into_iter().chain(std::iter::once(c))
        })
        .collect()
}

/// Builds and returns a new [`Table`] pre-populated with a styled red header row
/// used to display non-compliant commits.
///
/// Columns: Short SHA | Commit Message (spans 2) | Reason
fn build_error_table_header() -> Table {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Short SHA")
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
        Cell::new("Commit Message")
            .with_style(Attr::ForegroundColor(color::RED))
            .with_style(Attr::Italic(true))
            .with_hspan(2), // Spans two columns for readability
        Cell::new("Reason")
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
    ]));
    table
}

/// Creates a single error [`Row`] representing a non-compliant commit.
///
/// # Arguments
/// - `sha`         – The short commit hash (e.g. `"a1b2c3d"`).
/// - `commit_msg`  – The raw commit message to display (will be wrapped).
/// - `wrap_width`  – Max characters per line before inserting a newline.
/// - `error`       – Human-readable description of why the commit failed validation.
fn build_error_row(sha: &str, commit_msg: &str, wrap_width: usize, error: &str) -> Row {
    Row::new(vec![
        Cell::new(sha)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
        Cell::new(&wrap_text(commit_msg, wrap_width))
            .with_style(Attr::ForegroundColor(color::RED))
            .with_style(Attr::Italic(true))
            .with_hspan(2),
        Cell::new(error)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::RED)),
    ])
}

/// Checks whether recent git commits conform to the Conventional Commits specification.
///
/// Runs `git log` with a machine-readable format, parses each commit, and validates
/// it against the project [`Config`]. Any malformed commits are collected in an error
/// table that can optionally be printed and returned as an [`Err`].
///
/// # Arguments
/// - `name`        – Optional list of branch names/refs to pass to `git log`. When `None`, the current branch's full history is checked.
///
/// - `return_n`    – Optional override for the cell wrap width (reuses the param name from the CLI; defaults to [`DEFAULT_WRAP_WIDTH`] when `None`).
///
/// - `config`      – Project-level configuration used during commit parsing.
///
/// - `raise_error` – When `true`, returns an [`Err`] if any non-compliant commits are found (useful for CI/pre-push hooks). When `false`, the error table is still built but errors are silently ignored.
///
/// # Returns
/// `Ok((commits, err_table))` on success, where:
/// - `commits`   is the list of valid [`ConventionalCommit`]s found.
/// - `err_table` is a [`Table`] of validation failures (may be empty apart from header).
///
/// # Errors
/// - Returns [`Err`] if `raise_error` is `true` and non-compliant commits exist.
/// - Returns [`Err`] if `git log` produced zero parseable commits (likely a bad ref).
/// - Returns [`Err`] if the git output contains invalid UTF-8.
pub fn check(
    name: Option<Vec<String>>,
    return_n: Option<usize>,
    config: &Config,
    raise_error: bool,
) -> anyhow::Result<(Vec<ConventionalCommit>, Table)> {
    println!("\n{}\n# CHECK\n{}", SEPARATOR, SEPARATOR);

    // Use `return_n` as the cell wrap width, falling back to the default constant.
    let wrap_width = return_n.unwrap_or(DEFAULT_WRAP_WIDTH);

    // `format_args` produces a parseable, decoration-free log for processing.
    // `display_args` produces a human-readable graph for terminal output.
    let mut format_args: Vec<&str> = vec!["--no-decorate", "--format=\"%h%n%B\""];
    let mut display_args: Vec<&str> = vec!["--oneline", "--decorate", "--graph"];

    // Run `git log` with branch filters if provided, otherwise use the default ref.
    // In both cases, print the decorated graph first so the user can see the context.
    let (output, debug_cmd) = match name {
        Some(ref branches) => {
            let branch_args: Vec<&str> = branches.iter().map(String::as_str).collect();
            display_args.extend(&branch_args);
            format_args.extend(&branch_args);

            print!("LOG:\n{}", String::from_utf8(git::log(&display_args)?.stdout)?);
            let cmd_str = format_args.join(" ");
            (git::log(&format_args)?, cmd_str)
        }
        None => {
            print!("LOG:\n{}", String::from_utf8(git::log(&display_args)?.stdout)?);
            let cmd_str = format_args.join(" ");
            (git::log(&format_args)?, cmd_str)
        }
    };

    // Decode the raw bytes from git into a UTF-8 string for parsing.
    let raw = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("git output was not valid UTF-8: {}", e))?;
    debug!("{:#?}", raw);

    let mut commits: Vec<ConventionalCommit> = Vec::new();
    let mut err_table = build_error_table_header();

    // Each commit block is delimited by `"\n` (the closing quote of the format string).
    // Split on that boundary and parse each block into a (sha, message) pair.
    for row in raw.split("\"\n") {
        // An empty row signals the end of output; stop processing.
        if row.is_empty() {
            break;
        }

        // The format `%h%n%B` yields:  <short-sha>\n<body>
        // A missing newline means the block is malformed; skip it with a debug note.
        let Some((raw_sha, commit_msg)) = row.split_once('\n') else {
            debug!("skipping malformed row: {:?}", row);
            continue;
        };

        // Strip the leading `"` that git appends due to the quoted format string.
        let sha = raw_sha.replace('"', "");

        // Attempt to parse the commit as a Conventional Commit.
        // Valid commits are collected; invalid ones are added to the error table.
        match ConventionalCommit::new(commit_msg, config, sha.clone()) {
            Ok(commit) => commits.push(commit),
            Err(err) => {
                err_table.add_row(build_error_row(&sha, commit_msg, wrap_width, &err.to_string()));
            }
        }
    }

    // `err_table` always contains at least one row (the header), so `len() > 1`
    // means at least one non-compliant commit was found.
    if err_table.len() > 1 && raise_error {
        err_table.printstd();
        Err(anyhow::Error::msg(
            format!(
                "\nCommit History not cc compliant: \nfound {} bad commits out of {}",
                err_table.len() - 1,          // subtract the header row
                commits.len() + err_table.len() - 1 // total commits checked
            )
            .red(),
        ))
    } else if commits.is_empty() {
        // No commits were parsed at all — the ref/branch is likely invalid or empty.
        Err(anyhow::Error::msg(
            format!(
                "\ngit command error: \n0 commits were found from command:\n\t\t git log {}",
                debug_cmd
            )
            .red(),
        ))
    } else {
        info!("{}", format!("\n\nAll commits out of {} checked are ok", commits.len()).green());
        Ok((commits, err_table))
    }
}
