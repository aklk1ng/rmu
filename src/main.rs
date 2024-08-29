mod app;
mod parse;
mod term;
mod ui;

use color_eyre::Result;
use term::Term;

/// Main logical with tokio.
async fn tokio_main() -> Result<()> {
    let mut term = Term::new()?;
    Term::start()?;
    let res = ui::run(&mut term.terminal).await;
    Term::stop()?;
    if let Err(err) = res {
        Err(err)
    } else {
        Ok(())
    }
}

/// Main function.
#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
