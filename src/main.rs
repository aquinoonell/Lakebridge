pub mod arrow_schema;
pub mod archieve;
pub mod query_engine;
pub mod postgres;
pub mod cli;


use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::run(cli).await
}
