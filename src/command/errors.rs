#![allow(unused)]
mod check {
    use thiserror::Error;
    #[derive(Error, Debug, PartialEq)]
    pub enum CheckError {
        #[error("Commit History not CC compliant\n{0}")]
        CCNotCompliant(String),
    }
}
pub use check::CheckError;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("")]
    CheckError(#[from] check::CheckError),

    //TODO
    #[error("")]
    ReleaseError,
}
