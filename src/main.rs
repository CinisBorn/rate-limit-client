use crate::{cli::CliArgs, client::fetch};
use std::error::Error;

mod cli;
mod client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::get_args();

    let _ = fetch(args.endpoint, 10).await;
    Ok(())
}
