pub mod goxlr;
pub mod obs;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum AdapterCommand {
    Goxlr {
        #[command(subcommand)]
        command: goxlr::GoxlrCommand,
    },
    Obs {
        #[command(subcommand)]
        command: obs::ObsCommand,
    },
}

pub fn run(command: AdapterCommand) -> miette::Result<i32> {
    match command {
        AdapterCommand::Goxlr { command } => goxlr::run(command),
        AdapterCommand::Obs { command } => obs::run(command),
    }
}
