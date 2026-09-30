mod adapters;
mod core;
mod fsutil;
mod nix;
mod redaction;

use clap::{error::ErrorKind, Parser, Subcommand};
use miette::Result;

#[derive(Debug, Parser)]
#[command(name = "hermesix")]
#[command(about = "Generic managed configuration utilities")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Diff(core::Diff),
    Sync(core::Sync),
    Validate(core::Validate),
    Redact(redaction::Redact),
    Adapter {
        #[command(subcommand)]
        command: adapters::AdapterCommand,
    },
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let code = if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                0
            } else {
                2
            };
            let _ = err.print();
            std::process::exit(code);
        }
    };

    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{err:?}");
            std::process::exit(1);
        }
    }
}

fn run(cli: Cli) -> Result<i32> {
    match cli.command {
        Command::Diff(args) => core::diff_command(args),
        Command::Sync(args) => core::sync_command(args),
        Command::Validate(args) => {
            core::validate_command(args)?;
            Ok(0)
        }
        Command::Redact(args) => {
            redaction::redact_command(args)?;
            Ok(0)
        }
        Command::Adapter { command } => adapters::run(command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adapter_obs_export_to_nix() {
        Cli::try_parse_from(["hermesix", "adapter", "obs", "export-to-nix"])
            .expect("adapter OBS export command should parse");
    }

    #[test]
    fn parses_adapter_obs_plugin_inspect() {
        Cli::try_parse_from([
            "hermesix",
            "adapter",
            "obs",
            "plugin-inspect",
            "--source-dir",
            "./plugin",
        ])
        .expect("adapter OBS plugin inspect command should parse");
    }

    #[test]
    fn parses_adapter_obs_plugin_inspect_verify() {
        Cli::try_parse_from([
            "hermesix",
            "adapter",
            "obs",
            "plugin-inspect",
            "verify",
            "--evidence",
            "evidence.json",
            "--source-dir",
            "./plugin",
        ])
        .expect("adapter OBS plugin inspect verify command should parse");
    }

    #[test]
    fn rejects_old_obs_namespace() {
        let result = Cli::try_parse_from(["hermesix", "obs", "export-to-nix"]);
        assert!(result.is_err());
    }

    #[test]
    fn parses_adapter_goxlr_capture() {
        Cli::try_parse_from([
            "hermesix",
            "adapter",
            "goxlr",
            "capture",
            "--output-dir",
            "./goxlr-capture",
            "--json",
        ])
        .expect("adapter GoXLR capture command should parse");
    }
}
