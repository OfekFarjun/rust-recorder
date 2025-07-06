use anyhow::Error;
use log::{info, error};
use std::{
    io::Write, process::{Command, Stdio}, sync::{Arc, Mutex}, thread, time::Duration
};
use windows_capture::monitor::Monitor;

use crate::config::CaptureConfig;

pub fn ffmpeg_capture(
    recording: Arc<Mutex<bool>>,
    recording_raw: Arc<Mutex<bool>>,
    filename: String,
    capture_config: CaptureConfig,
) -> Result<(), anyhow::Error> {
    let primary_monitor = Monitor::primary()?;
    let width = primary_monitor.width()?;
    let height = primary_monitor.height()?;
    let fps = capture_config.fps;

    let child = Command::new("cmd")
        .args([
            "/C",
            &format!("ffmpeg.exe -video_size {width}x{height} -probesize 10M -f gdigrab -framerate {fps} -i desktop {filename}"),
        ])
        .stdin(Stdio::piped())
        .spawn()
        .or(Err(Error::msg("Could not start ffmpeg capture")))?;

    *recording.lock().unwrap() = true;
    *recording_raw.lock().unwrap() = true;
    info!("Starting capture via ffmpeg.exe");

    let child = Arc::new(Mutex::new(child));

    thread::spawn(move || {
        loop {
            if !*recording.lock().unwrap() {
                match (*child.lock().unwrap()).stdin.as_mut() {
                    Some(stdin) => {
                        info!("Writing q");
                        let _ = stdin.write_all("q".as_bytes());
                        stdin.flush().unwrap();
                        break;
                    },
                    None => {
                        error!("Could not stop ffmpeg.exe, stdin is not present");
                        ()
                    }
                };
                info!("Done with capture");
            }

            thread::sleep(Duration::from_millis(100));
        }

        let _ = child.lock().unwrap().wait();
        *recording.lock().unwrap() = false;
        *recording_raw.lock().unwrap() = false;
    });

    Ok(())
}
