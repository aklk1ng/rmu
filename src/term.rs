use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend as Backend;

/// Contain the ui terminal.
pub struct Term {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
}

impl Term {
    /// Create the `Term` structure.
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        Ok(Self { terminal })
    }

    /// Start the terminal raw mode and enable some features.
    pub fn start(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        set_panic_hook();
        Ok(())
    }

    /// Restore terminal to normal.
    pub fn restore() -> Result<()> {
        crossterm::execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
}

/// Register the hook when the panic happened in running.
fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = Term::restore(); // ignore any errors as we are already failing
        hook(panic_info);
    }));
}
