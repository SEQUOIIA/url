use std::env::VarError;
use std::io::{Read, Write};
use std::path::PathBuf;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_log_level")]
    pub log_level : String,
    #[serde(default = "default_api_key")]
    pub api_key : Option<String>,
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint : String,
}

pub fn default_log_level() -> String {
    return "INFO".to_owned()
}

pub fn default_api_key() -> Option<String> {
    return None
}

pub fn default_api_endpoint() -> String {
    return "http://localhost:8380".to_owned()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            api_key: default_api_key(),
            api_endpoint: default_api_endpoint()
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_file_path = match std::env::var("SEQ_URL_CONFIG") {
            Ok(val) => {
                std::path::PathBuf::from(val)
            },
            Err(_) => {
                let mut path = {
                    if cfg!(windows) {
                        dirs::config_dir().unwrap()
                    } else {
                        let mut p = dirs::home_dir().unwrap();
                        p.push(".config");
                        p
                    }
                };
                path.push("seqtf_url/config.yaml");
                path
            }
        };

        if !config_file_path.exists() {
            let default_conf = Self::default();
            let serialised = serde_yaml::to_vec(&default_conf)?;
            Self::write_conf_to_file(&config_file_path, &serialised)?;
        }

        let mut conf = std::fs::File::open(config_file_path)?;
        let mut buf = Vec::new();
        conf.read_to_end(&mut buf)?;
        Ok(serde_yaml::from_slice::<Config>(&buf)?)
    }

    fn write_conf_to_file(path : &PathBuf, context : &Vec<u8>) -> Result<()> {
        let dir_path = path.parent().unwrap();
        std::fs::create_dir_all(&dir_path)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(context)?;
        Ok(())
    }

    pub fn get_api_key(&self) -> Option<String> {
        match std::env::var("SEQ_URL_API_KEY") {
            Ok(val) => {
                return Some(val);
            }
            Err(_) => {}
        };

        if let Some(key) = self.api_key.as_ref() {
            return Some(key.clone());
        }

        None
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("unknown config error")]
    Unknown,
    #[error("config error: {0:?}")]
    CustomError(Box<dyn std::error::Error>),
    #[error("config error: {0}")]
    Message(String),
    #[error("io error: {0:?}")]
    IoError(std::io::Error),
    #[error("yaml error: {0:?}")]
    YamlError(serde_yaml::Error)
}