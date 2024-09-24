mod app;
mod config;
mod term;
mod ui;

use color_eyre::Result;

/// Main logical with tokio.
async fn tokio_main() -> Result<()> {
    color_eyre::install()?;
    ui::run().await?;
    Ok(())
}

/// Main function.
#[tokio::main]
async fn main() -> Result<()> {
    if !cfg!(target_os = "linux") {
        eprintln!("Use linux machine!!!");
    }

    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
