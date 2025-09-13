use std::process::ExitCode;

use clap::Args;

#[derive(Debug, Args)]
pub struct Parameters {
    /// The URL to your Tremolo orchestrator instance
    #[arg(long, env = "TREMOLO_HOST")]
    host: String,
}
pub fn start(_params: Parameters) -> ExitCode {
    println!("Hello, world!");

    ExitCode::SUCCESS
}
