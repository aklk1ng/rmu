use crate::{config, ui::*};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use rodio::{Decoder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

/// `Song` structure contains the name and total time about this song.
#[derive(Debug)]
pub struct Song {
    pub name: String,
    pub time: f64,
}

/// `App` contains all neccessary elements when running.
pub struct App<'a> {
    pub tabs: Tabstatus<'a>,
    pub progress: f64,
    pub tot_time: f64,
    pub cur_time: f64,
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
    pub async fn new(sink: Sink) -> App<'a> {
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
            tasks: StatefulList::with_items(config::playlist().await.unwrap()),
            tot_time: 0.0,
            cur_time: 0.0,
            sink,
            cur_idx: None,
            volume: 1.0,
            last_tick: Instant::now(),
            select_tick: Instant::now(),
        }
    }

    /// Select the previous song.
    pub fn select_up(&mut self) {
        self.tasks.previous();
    }

    /// Select the next song.
    pub fn select_down(&mut self) {
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

    /// The handle about the information needed.
    pub fn on_tick(&mut self) {
        self.set_progress();
        let value = self.barchart_data.pop().unwrap();
        self.barchart_data.insert(0, value);
    }

    /// Calculate the progress bar information.
    pub fn set_progress(&mut self) {
        match self.cur_idx {
            Some(i) => {
                let mut time = self.sink.get_pos().as_secs_f64();
                self.cur_time = time;
                self.tot_time = self.tasks.items[i].time;
                match self.tot_time {
                    0.0 => self.progress = 0.0,
                    _ => {
                        if time > self.tot_time {
                            time = self.tot_time
                        }
                        self.progress = time / self.tot_time;
                    }
                }
            }
            None => {
                self.cur_time = 0.0;
                self.tot_time = 0.0;
                self.progress = 0.0;
            }
        }
    }

    /// Clear the queue of the sounds to play.
    pub fn clear_list(&mut self) {
        self.sink.clear();
    }

    /// Get current selected song offset in `Song` Vec.
    fn get_offset(&mut self) -> usize {
        self.tasks.state.selected().unwrap_or(0)
    }

    /// Append a song to the queue of sounds to play.
    fn append_list(&mut self, offset: usize) {
        // let offset = self.get_offset();
        let file = BufReader::new(File::open(&self.tasks.items[offset].name).unwrap());
        let source = Decoder::new(file).unwrap();
        self.sink.append(source);
    }

    /// Add remaining songs to the queue of sounds to play.
    fn load_list(&mut self, offset: usize) {
        // Init the `self.cur_idx`.
        self.cur_idx = Some(offset);

        let n = self.tasks.items.len();
        for i in offset..n {
            self.append_list(i);
        }
        self.sink.play();
    }

    /// Now, when you press the enter to play this song, it will add remaining songs to the queue
    /// of sounds to play.
    pub fn play(&mut self) {
        self.sink.stop();
        let offset = self.get_offset();
        self.load_list(offset);
        self.tasks.state.select(Some(offset));
    }

    /// Replay current song.
    pub fn replay(&mut self) {
        match self.sink.try_seek(Duration::new(0, 0)) {
            Ok(_) => {}
            Err(e) => eprintln!("{}", e),
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
                    self.play();
                    self.start = true;
                }
                KeyCode::Char(c) => {
                    match c {
                        'q' => self.quit = true,
                        'h' => self.tab_left(),
                        'l' => self.tab_next(),
                        ' ' => self.toggle(),
                        'j' => self.select_down(),
                        'k' => self.select_up(),
                        'r' => self.replay(),
                        'e' => self.clear_list(),
                        '+' => self.increase_volume(),
                        '-' => self.decrease_volume(),
                        _ => {}
                    }
                    self.select_tick = Instant::now();
                }
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => self.select_down(),
                MouseEventKind::ScrollUp => self.select_up(),
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

            // Update the select song ui when the song play over.
            if let Some(cur_idx) = self.cur_idx {
                if self.tasks.items.len() - self.sink.len() > cur_idx {
                    self.cur_idx = Some(self.tasks.items.len() - self.sink.len());
                    self.tasks.state.select(self.cur_idx);
                }
            }
            self.on_tick();
            self.last_tick = Instant::now();
        }
    }

    /// Recover the select ui.
    pub fn recover_select(&mut self, tick_rate: Duration) {
        if self.select_tick.elapsed() >= tick_rate {
            if self.cur_idx.is_some() && (!self.sink.empty() || self.cur_idx.is_some()) {
                if let Some(i) = self.tasks.state.selected() {
                    if i != self.cur_idx.unwrap() {
                        self.tasks.state.select(self.cur_idx)
                    }
                }
            }
            self.select_tick = Instant::now();
        }
    }
}
