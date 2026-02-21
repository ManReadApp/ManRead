use helper::random_string;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub root_folder: PathBuf,
    pub port: u16,
    pub https_port: u16,
    pub rust_log: String,
    pub secret_key: String,
    pub spinner: Spinner,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct StorageConfig {
    #[serde(default = "default_true")]
    pub local: bool,
    #[serde(default)]
    pub encryption: bool,
    #[serde(default)]
    pub s3: S3Config,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    #[serde(default)]
    pub force_path_style: bool,
    #[serde(default)]
    pub upload_acl: S3UploadAclConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum S3UploadAclConfig {
    #[default]
    InheritBucket,
    Private,
    PublicRead,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: String::new(),
            endpoint: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            force_path_style: false,
            upload_acl: S3UploadAclConfig::InheritBucket,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            local: true,
            encryption: false,
            s3: S3Config::default(),
        }
    }
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
            port: 8082,
            https_port: 8081,
            rust_log: "info".to_string(),
            secret_key: random_string(64), //2048bit = 256byte = 64 chars
            spinner: Spinner::Pikachu2,
            storage: StorageConfig::default(),
        }
    }
}

const fn default_true() -> bool {
    true
}

pub fn get_env() -> std::io::Result<Config> {
    let path = PathBuf::from("Config.toml");
    if path.is_file() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        toml::from_str(&contents)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))
    } else {
        let config = Config::default();
        let serialized =
            toml::to_string(&config).map_err(|err| std::io::Error::other(err.to_string()))?;
        File::create(path)?.write_all(serialized.as_bytes())?;
        Ok(config)
    }
}
