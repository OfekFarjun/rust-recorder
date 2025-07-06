use config_file::FromConfigFile;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub recordings_folder: String,
    pub capture: CaptureConfig,
    pub keep_alive_timeout_in_secs: u64,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct CaptureConfig {
    pub ffmpeg: bool,
    pub bitrate: u32,
    pub fps: u32,
}

pub fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config::from_config_file("config.json")?)
}
