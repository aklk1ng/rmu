use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::*,
    Frame, Terminal,
};
use rodio::Decoder;
use rodio::{OutputStream, Sink};
use std::{
    fs::File,
    io::{self, BufReader},
    time::{Duration, Instant},
};

use crate::parse;

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
                if self.items.len() == 0 {
                    return;
                } else if i >= self.items.len() - 1 {
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
                if self.items.len() == 0 {
                    return;
                } else if i == 0 {
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

pub struct App<'a> {
    pub tabs: Tabstatus<'a>,
    pub progress: f64,
    pub quit: bool,
    pub barchart_data: Vec<(&'a str, u64)>,
    pub tasks: StatefulList<String>,
    pub messages: Vec<String>,
    pub playing_music: Option<Decoder<BufReader<File>>>,
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
            tasks: StatefulList::with_items(parse::playlist()),
            messages: Vec::new(),
            playing_music: None,
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

    fn load_playlist(&mut self, sink: &mut Sink) {
        for f in self.tasks.items.iter() {
            let file = BufReader::new(File::open(f.clone()).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
        }
    }

    fn music_play(&mut self, sink: &mut Sink) {
        let offset = self.tasks.state.selected().unwrap();
        let n = self.tasks.items.len();
        for i in offset..n {
            let file = BufReader::new(File::open(&self.tasks.items[i]).unwrap());
            let source = Decoder::new(file).unwrap();
            sink.append(source);
            sink.play();
        }
        // let file = BufReader::new(File::open(self.tasks.items.get(offset).unwrap()).unwrap());
        // let source = Decoder::new(file).unwrap();
        // sink.append(source);
        // sink.play();
    }

    fn key(&mut self, c: char, sink: &mut Sink) {
        match c {
            'q' => self.quit = true,
            'h' => self.to_left(),
            'l' => self.to_right(),
            ' ' => self.toggle(&sink),
            'j' => self.on_down(),
            'k' => self.on_up(),
            _ => {}
        }
    }
}

fn draw_gauge<B>(f: &mut Frame<B>, app: &App, chunk: Rect, sink: &Sink)
where
    B: Backend,
{
    let label = Span::styled(
        format!("{:.2}%", app.progress * 100.0),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::ITALIC | Modifier::BOLD),
    );
    // let position = sink.position().unwrap().as_secs() as f64;
    // let duration = sink.duration().unwrap().as_secs() as f64;
    // let progress = (position / duration) * 100.0;
    let gauge = Gauge::default()
        .block(Block::default().title("Progress").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .label(label)
        .ratio(app.progress)
        .use_unicode(true);
    f.render_widget(gauge, chunk);
}

fn draw_list<B>(f: &mut Frame<B>, app: &mut App, chunk: Rect)
where
    B: Backend,
{
    let tasks: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|item| item.split_at(item.rfind('/').unwrap() + 1).1)
        .map(|i| ListItem::new(vec![Line::from(Span::raw(i.to_owned()))]))
        .collect();
    let tasks = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(tasks, chunk, &mut app.tasks.state);
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, chunk: Rect, sink: &Sink)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(6), Constraint::Min(0)].as_ref())
        .split(chunk);
    draw_gauge(f, app, chunks[0], &sink);
    draw_list(f, app, chunks[1]);
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

pub fn ui<B>(f: &mut Frame<B>, app: &mut App, sink: &Sink)
where
    B: Backend,
{
    match app.tabs.index {
        0 => draw_first_tab(f, app, f.size(), &sink),
        1 => draw_second_tab(f, app, f.size()),
        _ => {}
    }
}

pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    mut sink: Sink,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app, &sink))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => app.music_play(&mut sink),
                    KeyCode::Char(c) => app.key(c, &mut sink),
                    _ => {}
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
