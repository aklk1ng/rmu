use color_eyre::Result;
use rodio::{Decoder, Source};
use serde_derive::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;

use crate::app::Song;
use crate::input::Input;

#[derive(Deserialize, Debug)]
pub struct Config {
    path: String,
}

impl Config {
    pub async fn new() -> Result<Self> {
        let crate_name = env!("CARGO_CRATE_NAME");

        let mut config_path = if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_dir)
        } else if let Ok(home_dir) = std::env::var("HOME") {
            let mut path = PathBuf::from(home_dir);
            path.push(".config");
            path
        } else {
            panic!("Neither XDG_CONFIG_HOME nor HOME environment variables are set");
        };
        config_path.push(crate_name);
        config_path.push("config.toml");

        match fs::read_to_string(&config_path).await {
            Ok(contents) => {
                let mut config: Config = toml::from_str(&contents)?;
                config.path = expand_var(&config.path);
                Ok(config)
            }
            Err(_) => {
                let path = Path::new(&config_path);
                if !path.exists() {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).await?;
                    }
                }

                // In this case, create a box to input config file path.
                let mut input = Input::new();
                match input.run() {
                    Ok(_) => {
                        let content = format!("path = \"{}\"", input.path);
                        fs::write(config_path, content.as_bytes()).await?;
                        let path = expand_var(&input.path);
                        Ok(Self { path })
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

/// Replace environment variable.
fn expand_var(path: &str) -> String {
    // Expand `~` variable.
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1).to_string();
        }
    }
    path.to_string()
}

pub async fn playlist() -> Result<Vec<Song>> {
    let mut playlist = Vec::new();
    let config = Config::new().await.unwrap();

    // Get all songs name
    let mut dir = fs::read_dir(config.path).await.unwrap();
    while let Ok(entry) = dir.next_entry().await {
        match entry {
            Some(entry) => {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "mp3" || extension == "mp4" || extension == "wav" {
                        let file = File::open(&path)?; // Replace with your file path
                        let source = Decoder::new(BufReader::new(file)).unwrap();
                        let total_duration =
                            source.total_duration().unwrap_or(Duration::from_secs(0));
                        let time = total_duration.as_secs_f64();
                        playlist.push(Song {
                            name: String::from(path.to_str().unwrap()),
                            time,
                        });
                    }
                }
            }
            None => break,
        }
    }
    Ok(playlist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_playlist() {
        let entries = playlist().await;
        for entry in entries.unwrap() {
            println!("{:?}", entry);
        }
    }
}
