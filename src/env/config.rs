use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub const ID_CHARS: [char; 71] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z', 'ä', 'Ä', 'ö', 'Ö', 'ü', 'Ü', '!', '?', '=',
];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub root_folder: PathBuf,
    pub port: u32,
    pub https_port: u32,
    pub rust_log: String,
    pub secret_key: String,
    pub spinner: Spinner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Spinner {
    Pikachu1,
    Pikachu2,
    Pikachu3,
    DragonBalls1,
    Ferris,
    Naruto1,
    NarutoEye1,
    Luffy1,
    Totoro1,
    Totoro2,
    Custom(String),
}

impl From<Spinner> for PathBuf {
    fn from(value: Spinner) -> PathBuf {
        let base = PathBuf::from("spinners");
        match value {
            Spinner::Pikachu1 => base.join("pikachu-running-1.gif"),
            Spinner::Pikachu2 => base.join("pikachu-running-2.gif"),
            Spinner::Pikachu3 => base.join("pikachu-running-3.gif"),
            Spinner::DragonBalls1 => base.join("dragon-ball-1.gif"),
            Spinner::Ferris => base.join("ferris.gif"),
            Spinner::Naruto1 => base.join("naruto-2.gif"),
            Spinner::NarutoEye1 => base.join("naruto-1.gif"),
            Spinner::Luffy1 => base.join("one-piece-1.gif"),
            Spinner::Totoro1 => base.join("totoro-1.gif"),
            Spinner::Totoro2 => base.join("totoro-2.gif"),
            Spinner::Custom(img) => base.join(img),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_folder: "data".into(),
            port: 8080,
            https_port: 8081,
            rust_log: "info".to_string(),
            secret_key: random_string(64), //2048bit = 256byte = 64 chars
            spinner: Spinner::Pikachu2,
        }
    }
}

pub fn random_string(len: usize) -> String {
    nanoid!(len, &ID_CHARS)
}

pub fn get_env() -> std::io::Result<Config> {
    let path = PathBuf::from("config.yml");
    if path.is_file() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_yaml::from_str(&contents).expect("Unable to deserialize YAML"))
    } else {
        let config = Config::default();
        File::create(path)?.write_all(
            serde_yaml::to_string(&config)
                .expect("Unable to serialize")
                .as_bytes(),
        )?;
        Ok(config)
    }
}
