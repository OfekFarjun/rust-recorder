use std::sync::{Arc, Mutex};

use crate::{config::CaptureConfig, ffmpeg, native_capture};

pub fn capture_screen(
    recording: Arc<Mutex<bool>>,
    recording_raw: Arc<Mutex<bool>>,
    filename: String,
    capture_config: CaptureConfig,
) -> Result<(), anyhow::Error> {
    if capture_config.ffmpeg {
        ffmpeg::capture::ffmpeg_capture(recording, recording_raw, filename, capture_config)?;
    } else {
        native_capture::record_screen(recording, recording_raw, filename, capture_config)?;
    }

    Ok(())
}
