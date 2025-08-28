use serde::{Serialize, Deserialize};

const CONFIG_PATH: &str = "portable_config/syncwatch.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub enable_on_start: bool,
    pub server_url: String,
    pub name: String,
    pub room_name: String,
}

impl Config {
    pub fn get() -> Option<Self> {
        let path = std::env::current_exe()
            .ok()
            .and_then(|exe_path| exe_path.parent().map(|parent| parent.join(CONFIG_PATH)))?;

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