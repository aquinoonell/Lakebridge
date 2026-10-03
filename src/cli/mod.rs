
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "lakebridge")]

pub struct Cli {}

pub async fn run(_cli: Cli) -> Result<()> {
   println!("lakebridge");
   Ok(())
}
