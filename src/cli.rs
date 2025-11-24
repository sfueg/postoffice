use std::fs;

use clap::Parser;
use serde::Deserialize;

use crate::{block::BlockConfig, connector::ConnectorConfig};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = String::from("config.json"))]
    pub file: String,

    #[arg(long)]
    pub ignore_cycles: bool,

    #[arg(long)]
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub connectors: Vec<ConnectorConfig>,
    pub blocks: Vec<BlockConfig>,
}

pub fn get_config(args: &Args) -> anyhow::Result<Config> {
    let config_data = fs::read_to_string(args.file.as_str())?;
    return Ok(serde_json::from_str::<Config>(&config_data)?);
}
