use clap::ArgAction;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct RuvexArgs {
    /// Name of the person to greet
    #[command(subcommand)]
    pub command: Option<RuvexCommand>,

    #[arg(short, long)]
    pub config_path: Option<String>,

    #[arg(short, long)]
    pub dry_run: bool,

    #[arg(long)]
    /// create a default config file in $HOME/.config
    pub create_default: bool,
}

#[derive(Subcommand, Debug)]
pub enum RuvexCommand {
    ///check if commit are CC compliant
    Check {
        ///branch or tag name to check for CC compliance
        #[arg()]
        name: Option<String>,

        ///branch or a tag to diff from
        #[arg(short, long)]
        diff: Option<String>,

        ///format error table by returning at nth char (default 40)
        #[arg(short, long)]
        format: Option<usize>,
    },
}
