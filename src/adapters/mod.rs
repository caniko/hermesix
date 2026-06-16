pub mod obs;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum AdapterCommand {
    Obs {
        #[command(subcommand)]
        command: obs::ObsCommand,
    },
}

pub fn run(command: AdapterCommand) -> miette::Result<i32> {
    match command {
        AdapterCommand::Obs { command } => obs::run(command),
    }
}
