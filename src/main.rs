use anyhow::Ok;
use clap::Parser;
use env_logger::Env;
use log::{debug, warn};
use ruvex::config::Config;
use ruvex::utils::cli::{RuvexArgs, RuvexCommand};
use std::path::Path;

fn create_default_config_file(config_path: &Path) -> anyhow::Result<()> {
    debug!("trying to create default path {:#?}", config_path);
    let dir_path = config_path.parent().unwrap();
    std::fs::create_dir_all(dir_path)?;
    Config::write_default(config_path.to_str().unwrap());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "info");

    env_logger::init_from_env(env);
    // value is if they're missing

    // Parse CLI args
    let args = RuvexArgs::parse();
    debug!("{:#?}", args);

    let default_config_path = Path::new(concat!(env!("HOME"), "/.config/ruvex/ruvex.yaml"));

    // check if config path set on RUVEX_CONFIG_PATH
    let config_path = match option_env!("RUVEX_CONFIG_PATH") {
        Some(value) => Path::new(value),
        None => {
            warn!(
                "RUVEX_CONFIG_PATH not set, will be defaulted to {}",
                default_config_path.to_str().unwrap()
            );
            default_config_path
        }
    };

    // create default config file if asked
    if args.create_default {
        create_default_config_file(&config_path)?;
    }

    //Init Config
    let config = Config::new(
        &args
            .config_path
            .unwrap_or(config_path.to_str().unwrap().to_owned()),
        default_config_path.to_str().unwrap(),
    )?;
    //Check Config
    config.config_check()?;

    debug!("config is {:#?}", config);
    match args.command {
        Some(RuvexCommand::Check { name, format }) => {
            ruvex::command::check(name, format, &config, true)?;
        }
        Some(RuvexCommand::Tag {
            merged,
            no_merged,
            ignore_prereleases,
            name,
        }) => ruvex::command::tag(name, merged, no_merged, ignore_prereleases, &config)?,
        _ => {}
    }
    Ok(())
}
