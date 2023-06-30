use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rodio::{Decoder, OutputStream, Sink};
use std::{
    error::Error,
    ffi::{OsStr, OsString},
    fs::{self, File},
    io::{self, BufReader},
    path::Path,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::parse;

const PATH: &Path = Path::new("/home/cjh/yt-dlp/music/");

const TASKS: [&str; 24] = [
    "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
    "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
    "Item20", "Item21", "Item22", "Item23", "Item24",
];

pub struct Tabstatus<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> Tabstatus<'a> {
    fn new(titles: Vec<&'a str>) -> Tabstatus {
        Tabstatus { titles, index: 0 }
    }

    fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len()
    }

    fn previous(&mut self) {
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
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub enum InputMode {
    Normal,
    Editing,
}
pub struct App<'a> {
    pub tabs: Tabstatus<'a>,
    pub progress: f64,
    pub quit: bool,
    pub barchart_data: Vec<(&'a str, u64)>,
    pub tasks: StatefulList<&'a str>,
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<String>,
    pub show_popup: bool,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            tabs: Tabstatus::new(vec!["Tab1", "Tab2"]),
            progress: 0.0,
            quit: false,
            barchart_data: vec![
                ("B1", 9),
                ("B2", 12),
                ("B3", 5),
                ("B4", 8),
                ("B5", 2),
                ("B6", 4),
                ("B7", 5),
                ("B8", 9),
                ("B9", 14),
                ("B10", 15),
                ("B11", 1),
                ("B12", 0),
                ("B13", 4),
                ("B14", 6),
                ("B15", 4),
                ("B16", 6),
                ("B17", 4),
                ("B18", 7),
                ("B19", 13),
                ("B20", 8),
                ("B21", 11),
                ("B22", 9),
                ("B23", 3),
                ("B24", 5),
            ],
            // tasks: StatefulList::with_items(TASKS.to_vec()),
            tasks: StatefulList::with_items(parse::playlist(PATH).into()),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            show_popup: false,
        }
    }

    fn on_up(&mut self) {
        self.tasks.previous();
    }

    fn on_down(&mut self) {
        self.tasks.next();
    }
    fn to_right(&mut self) {
        self.tabs.next()
    }

    fn to_left(&mut self) {
        self.tabs.previous()
    }

    fn toggle(&mut self, sink: &Sink) {
        if sink.is_paused() {
            sink.play()
        } else {
            sink.pause()
        }
    }

    fn on_tick(&mut self) {
        self.progress += 0.001;
        if self.progress > 1.0 {
            self.progress = 0.0
        }
        let value = self.barchart_data.pop().unwrap();
        self.barchart_data.insert(0, value);
    }

    fn key(&mut self, c: char, sink: &Sink) {
        match c {
            'q' => self.quit = true,
            'i' => self.input_mode = InputMode::Editing,
            'h' => self.to_left(),
            'l' => self.to_right(),
            ' ' => self.toggle(&sink),
            'j' => self.on_down(),
            'k' => self.on_up(),
            'p' => self.show_popup = !self.show_popup,
            _ => {}
        }
    }
}

fn draw_gauge<B>(f: &mut Frame<B>, app: &App, chunk: Rect)
where
    B: Backend,
{
    let label = Span::styled(
        format!("{:.2}%", app.progress * 100.0),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::ITALIC | Modifier::BOLD),
    );
    let gauge = Gauge::default()
        .block(Block::default().title("Gauge1").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .label(label)
        .ratio(app.progress)
        .use_unicode(true);
    f.render_widget(gauge, chunk);
}

fn draw_messages<B>(f: &mut Frame<B>, app: &mut App, chunk: Rect)
where
    B: Backend,
{
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunk);
}

fn draw_list<B>(f: &mut Frame<B>, app: &mut App, chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(chunk);
    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[0]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[0].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[0].y + 1,
            )
        }
    }
    let tasks: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
        .collect();
    let tasks = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(tasks, chunks[1], &mut app.tasks.state);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(6), Constraint::Min(0)].as_ref())
        .split(chunk);
    draw_gauge(f, app, chunks[0]);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);
    draw_list(f, app, chunks[0]);
    draw_messages(f, app, chunks[1]);

    if app.show_popup {
        let block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, f.size());
        f.render_widget(block, area);
    }
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &App, chunk: Rect)
where
    B: Backend,
{
    let barchart = BarChart::default()
        .block(Block::default().title("Data1").borders(Borders::ALL))
        .data(&app.barchart_data)
        .bar_width(9)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(barchart, chunk);
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.tabs.index {
        0 => draw_first_tab(f, app, f.size()),
        1 => draw_second_tab(f, app, f.size()),
        _ => {}
    }
}

pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    sink: Sink,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char(c) => app.key(c, &sink),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            app.messages.push(app.input.drain(..).collect());
                        }
                        KeyCode::Char(c) => app.input.push(c),
                        _ => {}
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.quit {
            return Ok(());
        }
    }
}
