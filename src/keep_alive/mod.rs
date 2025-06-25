use std::{
    sync::{Arc},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use log::info;

use crate::api::AppState;

pub fn keep_alive_task(state: Arc<AppState>) {
    thread::spawn(move || loop {
        let recording = *state.recording.lock().unwrap();
        let last_keep_alive = *state.last_keep_alive.lock().unwrap();
        thread::sleep(Duration::from_secs(1));
        let time_since_keep_alive = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - last_keep_alive;
        if recording && time_since_keep_alive >= state.config.keep_alive_timeout_in_secs {
            *state.recording.lock().unwrap() = false;
            info!("Closing screen capture due to lack of keep-alives!");
        }
    });
}
