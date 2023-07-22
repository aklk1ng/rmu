use anyhow::Result;
use serde_derive::Deserialize;
use std::fs::{self, File};
use std::io::Read;
use toml;

#[derive(Deserialize)]
pub struct Config {
    path: String,
}

fn parse_toml() -> Result<Config> {
    let home_dir = std::env::var("HOME")?;
    let mut config_file = File::open(format!("{home_dir}/.config/Music_Player/config.toml"))?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;
    let data: Config = toml::from_str(&contents)?;
    Ok(data)
}

pub fn playlist() -> Vec<String> {
    let mut playlist = Vec::new();

    let path = parse_toml().unwrap().path;

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "mp3" || extension == "mp4" || extension == "wav" {
                playlist.push(String::from(path.to_str().unwrap()));
            }
        }
    }
    playlist
}

mod tests {
    use super::*;
    #[test]
    fn test_play_list() {
        let entries = playlist();
        for entry in entries {
            println!("{:?}", entry);
        }
    }
}
