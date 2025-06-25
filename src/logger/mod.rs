extern crate pretty_env_logger;
use std::io::Write;
use chrono::Local;

pub fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::formatted_builder().format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] -> {}",
            Local::now().format("%d-%m-%Y %H:%M:%S").to_string(),
            record.level(),
            record.args()
        )
    }).filter(None, log::LevelFilter::Info).target(pretty_env_logger::env_logger::Target::Stdout).init();
}
