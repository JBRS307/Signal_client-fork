use clap::{crate_version, Parser, Subcommand, ValueEnum};

/// JSON-RPC client for signal-cli
#[derive(Parser, Debug)]
#[command(rename_all = "kebab-case", version = crate_version!())]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommands,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
#[command(rename_all = "camelCase", version = crate_version!())]
pub enum CliCommands {
    Link {
        #[arg(short = 'n', long)]
        name: String,
    },
    Version
}

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum LinkState {
    Enabled,
    EnabledWithApproval,
    Disabled,
}