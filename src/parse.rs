use serde_derive::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

#[derive(Deserialize)]
pub struct Data {
    config: Config,
}

#[derive(Deserialize)]
pub struct Config {
    path: String,
}

pub fn parse_toml() -> Config {
    let contents = fs::read_to_string("config.toml").unwrap();
    let data: Data = toml::from_str(&contents).unwrap();
    Config {
        path: data.config.path,
    }
}

pub fn playlist(path: &Path) -> Vec<PathBuf> {
    let mut playlist = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "mp3"
                || extension
                    == "mp4
                "
                || extension == "wav"
            {
                playlist.push(path);
            }
        }
    }
    playlist
}

mod tests {
    use super::*;
    #[test]
    fn test_play_list() {
        let entries = playlist(Path::new("/home/cjh/yt-dlp/music/"));
        for entry in entries {
            println!("{:?}", entry);
        }
    }
}
