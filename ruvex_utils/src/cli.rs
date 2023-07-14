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

    /// ignore prerelease tags when computing current version
    #[arg(short, long)]
    pub ignore_prereleases: bool,
}

#[derive(Subcommand, Debug)]
pub enum RuvexCommand {
    ///check if found commits are CC compliant, syntax is similar to git log
    Check {
        ///branch or tag name to check for CC compliance,
        ///"git log" sintax is supported e.g. branch..main
        #[arg(num_args(0..))]
        name: Option<Vec<String>>,

        ///format error table message by returning at nth char (default 40)
        #[arg(short, long)]
        format: Option<usize>,
    },
    Tag {
        #[arg(short, long)]
        merged: Option<String>,

        #[arg(short, long)]
        no_merged: Option<String>,

        #[arg(long)]
        ignore_prereleases: bool,

        #[arg(num_args(0..))]
        name: Option<Vec<String>>,
    },
}
