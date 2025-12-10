use crate::cli::Cli;
use std::time::Instant;
use tracing::{error, info};

mod cli;
mod kmeans;

fn main() {
    let start = Instant::now();
    let cli = Cli::new();

    // Setup logger.
    let logger = tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(false)
        .with_target(false);
    let logger = if cli.debug {
        logger.with_max_level(tracing::Level::DEBUG)
    } else {
        logger.with_max_level(tracing::Level::INFO)
    };
    logger.init();

    let res = cli.execute();
    match res {
        Ok(_) => {
            info!(ms = start.elapsed().as_millis(), "Processing completed");
        }
        Err(err) => {
            error!(error = err, "Error occurred");
        }
    }
}
