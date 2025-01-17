

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

use std::env;
use sc_cli::{SubstrateCli, CliConfiguration};
use sc_service::{Configuration, error::Error as ServiceError};
use non_traceable_chain_runtime::RuntimeApi;
use sc_executor::NativeExecutor;

pub struct Cli;
impl CliConfiguration for Cli {}

fn main() -> Result<(), ServiceError> {
    // Initialize the CLI and service configuration
    let cli = Cli::from_args();
    let config = cli.create_configuration()?;

    // Start the node service
    sc_service::new_full_startup::<RuntimeApi, _>(&config, NativeExecutor::<RuntimeApi>::new())?;

    // If you also need to run custom commands, call it here
    command::run()?;

    Ok(())
}
