use crate::{app::*, term::Term};
use color_eyre::Result;
use crossterm::event::{self};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::*,
    Frame,
};
use rodio::OutputStream;
use rodio::Sink;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

pub struct Tabstatus<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> Tabstatus<'a> {
    pub fn new(titles: Vec<&'a str>) -> Self {
        Tabstatus { titles, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len()
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1
        } else {
            self.index = self.titles.len() - 1
        }
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    self.items.len() - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

/// Draw the song progress bar.
fn draw_gauge(f: &mut Frame, app: &App, chunk: Rect) {
    let label = Span::styled(
        format!(
            "{:02}:{:02}/{:02}:{:02}",
            (app.cur_time / 60.0) as u64,
            (app.cur_time % 60.0) as u64,
            (app.tot_time / 60.0) as u64,
            (app.tot_time % 60.0) as u64,
        ),
        Style::default().fg(Color::Yellow),
    );
    let gauge = LineGauge::default()
        .filled_style(Style::default().fg(Color::Magenta))
        .line_set(symbols::line::THICK)
        .label(label)
        .ratio(app.progress);
    f.render_widget(gauge, chunk);
}

/// Draw all songs's name and time in a list.
fn draw_list(f: &mut Frame, app: &mut App, chunk: Rect) {
    let tasks: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|item| {
            let name = item.name.split_at(item.name.rfind('/').unwrap() + 1).1;
            let time = format!(
                "{:02}:{:02}",
                (item.time / 60.0) as u64,
                (item.time % 60.0) as u64
            );

            let padding = (chunk.width as usize)
                .saturating_sub(UnicodeWidthStr::width(name))
                .saturating_sub(time.len());
            ListItem::new(vec![Line::from(Span::raw(format!(
                "{}{}{}",
                name,
                " ".repeat(padding),
                time
            )))])
        })
        .collect();

    let tasks = List::new(tasks)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(tasks, chunk, &mut app.tasks.state);
}

/// Draw the first tab.
fn draw_first_tab(f: &mut Frame, app: &mut App, chunk: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(chunk);
    draw_gauge(f, app, chunks[0]);
    draw_list(f, app, chunks[1]);
}

/// Draw the second tab.
fn draw_second_tab(f: &mut Frame, app: &App, chunk: Rect) {
    let barchart = BarChart::default()
        .data(&app.barchart_data)
        .bar_width(7)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Yellow).bg(Color::Yellow));
    f.render_widget(barchart, chunk);
}

/// Main logic about ui.
pub fn ui(f: &mut Frame, app: &mut App) {
    match app.tabs.index {
        0 => draw_first_tab(f, app, f.area()),
        1 => draw_second_tab(f, app, f.area()),
        _ => {}
    }
}

/// Run the program, draw the terminal and handle the key pressed.
pub async fn run() -> Result<()> {
    let mut term = Term::new()?;
    term.start()?;
    let tick_rate = Duration::from_millis(200);
    let recover_delay = Duration::from_secs(3);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut app = App::new(sink).await;

    loop {
        term.terminal.draw(|f| ui(f, &mut app))?;
        let timeout = tick_rate
            .checked_sub(app.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            match event::read() {
                Ok(ev) => app.handle_events(ev),
                Err(e) => return Err(e.into()),
            }
        }

        // Just use timer to update special stuff.
        app.update(tick_rate);
        app.recover_select(recover_delay);

        if app.quit {
            break;
        }
    }

    Term::restore()?;
    Ok(())
}
