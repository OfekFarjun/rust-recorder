mod api;
mod capture;
mod config;
mod keep_alive;
mod logger;
mod audio;
mod ffmpeg;
mod native_capture;

use log::info;

#[tokio::main]
async fn main() {
    logger::init_logger();
    info!("The Rust-Recorder is listening on port {}", 3030);
    let config = config::get_config().expect("./config.json should be present and valid");
    std::fs::create_dir_all(&config.recordings_folder).expect("Could not create the recordings folder");
    api::start(config).await;
}
