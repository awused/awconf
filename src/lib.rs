use std::env::current_exe;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use toml::de;

#[derive(Debug)]
pub enum Error {
    /// An error when reading the file.
    IO(std::io::Error),
    /// An error during deserialization.
    Deserialization(de::Error),
    /// No suitable configuration file was found.
    NotFound,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<de::Error> for Error {
    fn from(e: de::Error) -> Self {
        Self::Deserialization(e)
    }
}

/// Attempts to load a config for the application with the given name, trying
/// files different locations in order of priority.
/// Can be overridden by specifying an override_name.
/// ./appname.toml
/// $HOME/.appname.toml
/// $XDG_CONFIG_HOME/appname/appname.toml
/// $XDG_CONFIG_HOME/appname/config.toml
/// /usr/local/etc/appname.toml
/// /usr/etc/appname.toml
/// <executable directory>/appname.toml
pub fn load_config<T: DeserializeOwned>(
    name: &str,
    override_name: &Option<PathBuf>,
) -> Result<T, Error> {
    if let Some(p) = override_name {
        return Ok(toml::from_slice(&std::fs::read(p)?)?);
    }

    let nametoml = format!("{}.toml", name);

    let mut paths = vec![PathBuf::from(&nametoml)];

    if let Some(h) = dirs::home_dir() {
        paths.push(h.join(".".to_owned() + &nametoml));
    }

    if let Some(c) = dirs::config_dir() {
        paths.push(c.join(name).join(&nametoml));
        paths.push(c.join(name).join("config.toml"));
    }

    paths.push(PathBuf::from("/usr/local/etc").join(&nametoml));
    paths.push(PathBuf::from("/user/etc").join(&nametoml));

    if let Ok(mut p) = current_exe() {
        p.pop();
        paths.push(p.join(&nametoml));
    }

    for p in paths {
        if !p.is_file() {
            continue;
        }

        return Ok(toml::from_slice(&std::fs::read(p)?)?);
    }

    Err(Error::NotFound)
}
