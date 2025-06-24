mod api;
mod capture;
mod config;
mod keep_alive;

#[tokio::main]
async fn main() {
    println!("\n\n The Rust-Recorder is listening on port {}\n\n", 3030);
    let config = config::get_config().expect("./config.json should be present and valid");
    std::fs::create_dir_all(&config.recordings_folder).expect("Could not create the recordings folder");
    api::start(config).await;
}
