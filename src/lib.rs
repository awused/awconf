use std::env::current_exe;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use serde::de::DeserializeOwned;
use toml::de;

#[derive(Debug)]
pub enum Error {
    /// An error when reading the file.
    IO(std::io::Error),
    /// An error during deserialization.
    Deserialization(de::Error),
    /// The TOML content was not valid UTF-8
    Utf8Error(std::str::Utf8Error),
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

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO error [{e}]"),
            Self::Deserialization(e) => write!(f, "Deserialization failed [{e}]"),
            Self::Utf8Error(e) => write!(f, "Invalid UTF-8 [{e}]"),
            Self::NotFound => write!(f, "No config found"),
        }
    }
}

/// Attempts to load a config for the application with the given name, trying
/// files different locations in order of priority. Returns the config and where it was loaded
/// from.
///
/// Can be overridden by specifying an `override_file` which should be exposed to end users as a
/// cli flag.
///
/// `default_conf`, if set, will be used when nothing can be found or when `override_file` is
/// neither a file nor a directory, such as /dev/null.
///
/// ./appname.toml
/// $HOME/.appname.toml
/// $XDG_CONFIG_HOME/appname/appname.toml
/// $XDG_CONFIG_HOME/appname/config.toml
/// /usr/local/etc/appname.toml
/// /usr/etc/appname.toml
/// [executable directory]/appname.toml
pub fn load_config<T: DeserializeOwned>(
    name: impl AsRef<str>,
    override_file: Option<impl AsRef<Path>>,
    default_conf: Option<impl AsRef<str>>,
) -> Result<(T, Option<PathBuf>), Error> {
    let name = name.as_ref();
    let default_conf = default_conf.as_ref().map(AsRef::as_ref);

    if let Some(path) = override_file {
        let p = path.as_ref();
        if p.exists() && !p.is_file() && !p.is_dir() {
            if let Some(default_conf) = default_conf {
                return Ok((toml::from_str(default_conf)?, None));
            }
        }

        return Ok((toml::from_str(from_utf8(&std::fs::read(p)?)?)?, Some(p.to_path_buf())));
    }

    let nametoml = format!("{name}.toml");

    let mut paths = vec![PathBuf::from(&nametoml)];

    if let Some(h) = dirs::home_dir() {
        paths.push(h.join(".".to_owned() + &nametoml));
    }

    if let Some(c) = dirs::config_dir() {
        paths.push(c.join(name).join(&nametoml));
        paths.push(c.join(name).join("config.toml"));
    }

    paths.push(PathBuf::from("/usr/local/etc").join(&nametoml));
    paths.push(PathBuf::from("/usr/etc").join(&nametoml));

    if let Ok(mut p) = current_exe() {
        p.pop();
        paths.push(p.join(&nametoml));
    }

    for p in paths {
        if !p.is_file() {
            continue;
        }

        return Ok((toml::from_str(from_utf8(&std::fs::read(&p)?)?)?, Some(p)));
    }

    if let Some(default_conf) = default_conf {
        return Ok((toml::from_str(default_conf)?, None));
    }

    Err(Error::NotFound)
}
