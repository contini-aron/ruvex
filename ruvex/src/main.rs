use clap::Parser;
use ruvex_commands;
use ruvex_config::Config;
use ruvex_utils::cli::{RuvexArgs, RuvexCommand};

fn main() -> anyhow::Result<()> {
    let args = RuvexArgs::parse();
    println!("{:?}", args);

    let default_ruvex_config_dir = concat!(env!("HOME"), "/.config/ruvex");
    let default_ruvex_config_path = &format!("{}/config.yaml", default_ruvex_config_dir);

    if args.create_default {
        println!(
            "trying to create default directory {}",
            default_ruvex_config_path
        );
        std::fs::create_dir_all(default_ruvex_config_dir)?;
        println!("done");
        Config::write_default(default_ruvex_config_path);
    } else {
        let ruvex_config_path = match option_env!("RUVEX_CONFIG_PATH") {
            Some(value) => value,
            None => {
                println!(
                "RUVEX_CONFIG_PATH not set, will be defaulted to $HOME/.config/ruvex/config.yaml"
                );
                default_ruvex_config_path
            }
        };
        let config = Config::new(
            &args.config_path.unwrap_or(ruvex_config_path.to_owned()),
            default_ruvex_config_path,
        )
        .unwrap();
        println!("{:?}", config);
        match args.command {
            Some(RuvexCommand::Check { name, diff, format }) => {
                ruvex_commands::check(name, diff, format, &config)?
            }
            _ => {}
        }
    }
    Ok(())
}
