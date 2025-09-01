use serde::{Serialize, Deserialize};

const CONFIG_NAME: &str = "syncwatch.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub enable_on_start: bool,
    pub server_url: String,
    pub name: String,
    pub room_name: String,
}

impl Config {
    pub fn get() -> Option<Self> {
        #[cfg(windows)]
        let path = {
            let executable_path = std::env::current_exe().ok()?;
            executable_path.parent()?.join("portable_config").join(CONFIG_NAME)
        };
        #[cfg(unix)]
        let path = {
            let home = std::env::home_dir()?;
            home.join(".config").join("mpv").join(CONFIG_NAME)
        };

        log::trace!("Looking for config file at: {:?}", path);

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read config file: {}", e);
                return None;
            }
        };

        let config = match toml::from_str::<Config>(&contents) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to parse config file: {}", e);
                return None;
            }
        };

        log::trace!("Loaded config: {:?}", config);

        Some(config)
    }
}