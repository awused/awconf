use std::env::current_exe;
use std::path::PathBuf;

use config::{self, Config, ConfigError, File};
use serde::Deserialize;

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
pub fn load_config<'a, T: Deserialize<'a>>(
    name: &str,
    override_name: &Option<String>,
) -> Result<T, ConfigError> {
    if let Some(p) = override_name {
        let mut c = Config::default();
        match c.merge(File::with_name(p)) {
            Ok(_) => return c.try_into(),
            Err(e) => return Err(e),
        }
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
        let mut c = Config::default();
        if c.merge(File::from(p)).is_ok() {
            return c.try_into();
        }
    }

    Err(ConfigError::NotFound("No config found.".to_string()))
}
