use color_eyre::Result;
use dirs::home_dir;
use rodio::{Decoder, Source};
use serde_derive::Deserialize;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::time::Duration;

use crate::app::Song;

#[derive(Deserialize, Debug)]
pub struct Config {
    path: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        let crate_name = env!("CARGO_CRATE_NAME");
        let home_dir = std::env::var("HOME")?;
        let mut config_file = File::open(format!("{home_dir}/.config/{crate_name}/config.toml"))?;
        let mut contents = String::new();
        config_file.read_to_string(&mut contents)?;
        let mut config: Config = toml::from_str(&contents)?;
        config.path = expand_var(&config.path);
        Ok(config)
    }
}

/// Replace the `~` to env `HOME` possible in config file path
fn expand_var(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = home_dir() {
            return path.replacen("~", home.to_str().unwrap(), 1);
        }
    }
    path.to_string()
}

pub fn playlist() -> Vec<Song> {
    let mut playlist = Vec::new();
    let config = Config::new().unwrap();

    // Get all songs name
    for entry in fs::read_dir(config.path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "mp3" || extension == "mp4" || extension == "wav" {
                let file = File::open(&path).unwrap(); // Replace with your file path
                let source = Decoder::new(BufReader::new(file)).unwrap();
                let total_duration = source.total_duration().unwrap_or(Duration::from_secs(0));
                let time = total_duration.as_secs_f64();
                playlist.push(Song {
                    name: String::from(path.to_str().unwrap()),
                    time,
                });
            }
        }
    }
    playlist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist() {
        let entries = playlist();
        for entry in entries {
            println!("{:?}", entry);
        }
    }
}
