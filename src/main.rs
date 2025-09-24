use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod agent;
mod server;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the worker agent to connect to the Tremolo orchestrator server
    Agent(agent::Parameters),
    /// Start the Tremolo orchestrator server
    Server(server::Parameters),
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Cli::parse();

    match args.command {
        Command::Agent(params) => agent::start(params).await,
        Command::Server(params) => server::start(params).await,
    }
}
