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

/// Replace the `~` to env `HOME` possible in config file path
fn expand_var(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = home_dir() {
            return path.replacen("~", home.to_str().unwrap(), 1);
        }
    }
    path.to_string()
}

/// Parse the toml contents to `Config` structure
fn parse_toml() -> Result<Config> {
    let home_dir = std::env::var("HOME")?;
    let mut config_file = File::open(format!("{home_dir}/.config/Music_Player/config.toml"))?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;
    let mut data: Config = toml::from_str(&contents)?;
    data.path = expand_var(&data.path);
    Ok(data)
}

pub fn playlist() -> Vec<Song> {
    let mut playlist = Vec::new();

    let path = match parse_toml() {
        Ok(config) => config.path,
        Err(_) => panic!("not config file!"),
    };

    // Get all songs name
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "mp3" || extension == "mp4" || extension == "wav" {
                let file = File::open(&path).unwrap(); // Replace with your file path
                let source = Decoder::new(BufReader::new(file)).unwrap();
                let total_duration = source.total_duration().unwrap_or(Duration::from_secs(0));
                let total_minutes = total_duration.as_secs() / 60;
                let total_seconds = total_duration.as_secs() % 60;
                playlist.push(Song {
                    name: String::from(path.to_str().unwrap()),
                    time: (total_minutes, total_seconds),
                });
            }
        }
    }
    playlist
}

#[test]
fn test_play_list() {
    let entries = playlist();
    for entry in entries {
        println!("{:?}", entry);
    }
}
