use color_eyre::Result;
use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
};
use ratatui::{
    layout::{Constraint, Layout, Position},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::term::Term;

/// Input structure hold the state about the config file input.
pub struct Input {
    /// Current value of the input box.
    input: String,
    /// Position of the cursor in the editor area.
    char_idx: usize,
    /// Submit input.
    pub path: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            char_idx: 0,
            path: String::new(),
        }
    }

    /// Move cursor left.
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_idx.saturating_sub(1);
        self.char_idx = self.clamp_cursor(cursor_moved_left);
    }

    /// Move cursor right.
    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_idx.saturating_add(1);
        self.char_idx = self.clamp_cursor(cursor_moved_right);
    }

    /// Move cursor to beginning.
    fn move_cursor_begin(&mut self) {
        self.char_idx = self.clamp_cursor(0);
    }

    /// Move cursor to end.
    fn move_cursor_end(&mut self) {
        self.char_idx = self.clamp_cursor(usize::MAX);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_idx)
            .unwrap_or(self.input.len())
    }

    /// Delete the char under cursor.
    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_idx != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.char_idx;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    /// Reset the cursor.
    fn reset_cursor(&mut self) {
        self.char_idx = 0;
    }

    fn submit(&mut self) {
        self.path = self.input.clone();
        self.input.clear();
        self.reset_cursor();
    }

    pub fn run(&mut self) -> Result<()> {
        let mut term = Term::new()?;
        set_bar_cursor();
        loop {
            term.terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('b'), KeyModifiers::CONTROL) => self.move_cursor_left(),
                        (KeyCode::Char('f'), KeyModifiers::CONTROL) => self.move_cursor_right(),
                        (KeyCode::Char('a'), KeyModifiers::CONTROL) => self.move_cursor_begin(),
                        (KeyCode::Char('e'), KeyModifiers::CONTROL) => self.move_cursor_end(),
                        (KeyCode::Enter, KeyModifiers::NONE) => {
                            self.submit();
                            break;
                        }
                        (KeyCode::Backspace, KeyModifiers::NONE) => self.delete_char(),
                        (KeyCode::Esc, KeyModifiers::NONE) => break,
                        (KeyCode::Char(to_insert), _) => self.enter_char(to_insert),
                        _ => {}
                    }
                }
            }
        }
        // Clear the screen
        term.terminal.clear()?;
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let text = vec![
            Line::from(vec![
                "Input config file path".blue(),
                "(are you forget to set the config file?)".yellow().bold(),
            ]),
            Line::from(vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to submit config file path".into(),
            ]),
        ];

        let vertical = Layout::vertical([
            // Just for padding
            Constraint::Percentage(40),
            Constraint::Length(text.len() as u16),
            Constraint::Length(3),
        ]);
        let [_, help_area, input_area] = vertical.areas(frame.area());

        let text = Text::from(text).patch_style(Style::default());
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default())
            .block(Block::bordered().title("Input"))
            .wrap(Wrap { trim: false });
        frame.render_widget(input, input_area);
        // Make the cursor visible and ask ratatui to put it at the specified coordinates after rendering
        #[allow(clippy::cast_possible_truncation)]
        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            input_area.x
                + self
                    .input
                    .chars()
                    .take(self.char_idx)
                    .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
                    .sum::<usize>() as u16
                + 1,
            input_area.y + 1,
        ))
    }
}

fn set_bar_cursor() {
    execute!(std::io::stdout(), SetCursorShape(CursorShape::Line)).unwrap();
}
