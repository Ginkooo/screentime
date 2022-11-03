use std::{fs::File, path::PathBuf};

use config::Config;

use crate::consts::{
    DEFAULT_PORT, DEFAULT_SECONDS_BEFORE_AFK, DEFAULT_SNAPSHOT_INTERVAL_IN_SECONDS, PORT,
    SECONDS_BEFORE_AFK, SNAPSHOT_INTERVAL_IN_SECONDS,
};

pub struct ScreentimeConfig {
    pub config: Config,
}

impl ScreentimeConfig {
    pub fn new(path: PathBuf) -> Self {
        let config = Self::make_config(path).expect("could not make config");
        ScreentimeConfig { config }
    }

    fn create_config_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(path.parent().ok_or("no parent")?)?;
        if !path.exists() {
            File::create(path)?;
        }
        Ok(())
    }

    fn make_config(path: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
        Self::create_config_file(&path)?;
        let config = Config::builder()
            .add_source(config::File::with_name(path.to_str().ok_or("wrong path")?))
            .set_default(PORT, DEFAULT_PORT)?
            .set_default(
                SNAPSHOT_INTERVAL_IN_SECONDS,
                DEFAULT_SNAPSHOT_INTERVAL_IN_SECONDS,
            )?
            .set_default(SECONDS_BEFORE_AFK, DEFAULT_SECONDS_BEFORE_AFK)?
            .build()?;
        Ok(config)
    }
}

impl Default for ScreentimeConfig {
    fn default() -> Self {
        let mut path = dirs::config_dir().expect("cannot create the config directory");
        path.push("screentime");
        path.push("config.toml");
        let config = Self::make_config(path).expect("could not make config");
        ScreentimeConfig { config }
    }
}
