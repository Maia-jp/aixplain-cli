#![allow(dead_code)]

mod api;
mod cli;
mod client;
mod config;
mod models;
mod tui;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::dispatch(cli).await
}
