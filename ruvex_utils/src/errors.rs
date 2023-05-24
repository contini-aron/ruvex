use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ConventionalCommitParseError {
    #[error("Missing \":\"")]
    MissingColumn,

    #[error("Empty scope ()")]
    EmptyScope,

    #[error("Commit message does not have a space after :")]
    NoSpaceAfterColumn,

    #[error("invalid type\n(expected {expected:?},\nfound {found:?})")]
    InvalidType {
        expected: Vec<String>,
        found: String,
    },
}
