use crate::{parse, ui::*};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use rodio::{Decoder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

/// `Song` structure contains the name and total time about this song.
#[derive(Debug)]
pub struct Song {
    pub name: String,
    pub time: (u64, u64),
}

/// `App` contains all neccessary elements when running.
pub struct App<'a> {
    pub tabs: Tabstatus<'a>,
    pub progress: f64,
    pub tot_time: (u64, u64),
    pub cur_time: (u64, u64),
    pub cur_idx: Option<usize>,
    pub start: bool,
    pub quit: bool,
    pub barchart_data: Vec<(&'a str, u64)>,
    pub tasks: StatefulList<Song>,
    pub sink: Sink,
    pub volume: f32,
    pub last_tick: Instant,
    pub select_tick: Instant,
}

impl<'a> App<'a> {
    /// Create the `App`
    pub fn new(sink: Sink) -> App<'a> {
        App {
            tabs: Tabstatus::new(vec!["Tab1", "Tab2"]),
            progress: 0.0,
            quit: false,
            start: false,
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
            tot_time: (0, 0),
            cur_time: (0, 0),
            sink,
            cur_idx: None,
            volume: 1.0,
            last_tick: Instant::now(),
            select_tick: Instant::now(),
        }
    }

    /// Select the previous song.
    pub fn on_up(&mut self) {
        self.tasks.previous();
    }

    /// Select the next song.
    pub fn on_down(&mut self) {
        self.tasks.next();
    }

    /// Switch the next tab.
    pub fn tab_next(&mut self) {
        self.tabs.next()
    }

    /// Switch the previous tab.
    pub fn tab_left(&mut self) {
        self.tabs.previous()
    }

    /// Toggle whether pauses playback of this sink.
    pub fn toggle(&mut self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    /// The handle about the `App.progress`.
    pub fn on_tick(&mut self) {
        self.set_progress();
        let value = self.barchart_data.pop().unwrap();
        self.barchart_data.insert(0, value);
    }

    /// Recalculate the progress bar information.
    pub fn set_progress(&mut self) {
        match self.cur_idx {
            Some(i) => {
                let time = self.sink.get_pos();
                let minutes = time.as_secs() / 60;
                let seconds = time.as_secs() % 60;
                self.cur_time = (minutes, seconds);
                self.tot_time = self.tasks.items[i].time;
                match self.tot_time {
                    (0, 0) => self.progress = 0.0,
                    _ => {
                        self.progress = (self.cur_time.0 * 60 + self.cur_time.1) as f64
                            / (self.tot_time.0 * 60 + self.tot_time.1) as f64;
                    }
                }
            }
            None => {
                self.cur_time = (0, 0);
                self.tot_time = (0, 0);
                self.progress = 0.0;
            }
        }
    }

    /// Now, when you press the enter to play this song, it will add remaining songs to the queue
    /// of sounds to play.
    pub fn music_play(&mut self) {
        self.sink.stop();
        let offset = match self.tasks.state.selected() {
            Some(n) => n,
            // In this case, just select the first song.
            None => {
                self.tasks.state.select(Some(0));
                0
            }
        };
        // Init the `self.cur_idx`.
        self.cur_idx = Some(offset);
        // Add remaining songs to the list.
        let n = self.tasks.items.len();
        for i in offset..n {
            let file = BufReader::new(File::open(&self.tasks.items[i].name).unwrap());
            let source = Decoder::new(file).unwrap();
            self.sink.append(source);
            self.sink.play();
        }
    }

    /// Increase the volume of the sound.
    pub fn increase_volume(&mut self) {
        self.volume += 0.2;
        self.sink.set_volume(self.volume);
    }

    /// Decrease the volume of the sound.
    pub fn decrease_volume(&mut self) {
        self.volume -= 0.2;
        self.sink.set_volume(self.volume);
    }

    /// Handle the key and mouse events.
    pub fn handle_events(&mut self, ev: Event) {
        match ev {
            Event::Key(key) => match key.code {
                KeyCode::Enter => {
                    self.music_play();
                    self.start = true;
                }
                KeyCode::Char(c) => {
                    match c {
                        'q' => self.quit = true,
                        'h' => self.tab_left(),
                        'l' => self.tab_next(),
                        ' ' => self.toggle(),
                        'j' => self.on_down(),
                        'k' => self.on_up(),
                        '+' => self.increase_volume(),
                        '-' => self.decrease_volume(),
                        _ => {}
                    }
                    self.select_tick = Instant::now();
                }
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => self.on_down(),
                MouseEventKind::ScrollUp => self.on_up(),
                _ => {}
            },
            _ => {}
        }
    }

    /// Update all components in `App`.
    pub fn update(&mut self, tick_rate: Duration) {
        if self.last_tick.elapsed() >= tick_rate {
            if self.start && self.sink.empty() {
                self.tasks.state.select(None);
                self.cur_idx = None;
                self.start = false;
            }

            self.on_tick();
            self.last_tick = Instant::now();
            // Restore the select ui.
            if self.select_tick.elapsed() >= Duration::from_secs(3) {
                if let Some(cur_idx) = self.cur_idx {
                    // Update the select song ui when the song play over.
                    if self.tasks.items.len() - self.sink.len() > cur_idx {
                        self.cur_idx = Some(self.tasks.items.len() - self.sink.len());
                        self.tasks.state.select(self.cur_idx);
                    }

                    // Check current selected item whether is current play.
                    if !self.sink.empty() || self.cur_idx.is_some() {
                        if let Some(i) = self.tasks.state.selected() {
                            if i != self.cur_idx.unwrap() {
                                self.tasks.state.select(self.cur_idx)
                            }
                        }
                    }
                }
                self.select_tick = Instant::now();
            }
        }
    }
}
