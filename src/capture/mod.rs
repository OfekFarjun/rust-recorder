use anyhow::Ok;
use log::{info, warn};
use std::{
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    encoder::{AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptureSettings, DrawBorderSettings, Settings},
};

use crate::config::CaptureConfig;
// Handles capture events.
struct Capture {
    // The video encoder that will be used to encode the frames.
    encoder: Option<VideoEncoder>,
    flags: CustomFlags,
}

#[derive(Debug)]
struct CustomFlags {
    width: u32,
    height: u32,
    filename: Arc<String>,
    capture_config: CaptureConfig,
    recording_raw: Arc<Mutex<bool>>,
}

impl GraphicsCaptureApiHandler for Capture {
    // The type of flags used to get the values from the settings.
    type Flags = CustomFlags;

    // The type of error that can be returned from `CaptureControl` and `start` functions.
    type Error = anyhow::Error;

    // Function that will be called to create a new instance. The flags can be passed from settings.
    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        info!(
            "Capture is starting via the windows-capture api, with flags: {:?}",
            ctx.flags
        );

        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(ctx.flags.width, ctx.flags.height)
                .bitrate(ctx.flags.capture_config.bitrate)
                .frame_rate(ctx.flags.capture_config.fps),
            AudioSettingsBuilder::default().disabled(true),
            ContainerSettingsBuilder::default(),
            Path::new(&*ctx.flags.filename.as_str()),
        )?;

        Ok(Self {
            encoder: Some(encoder),
            flags: ctx.flags,
        })
    }

    // Called every time a new frame is available.
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // Send the frame to the video encoder
        self.encoder.as_mut().unwrap().send_frame(frame)?;

        *self.flags.recording_raw.clone().lock().unwrap() = true;

        // Note: The frame has other uses too, for example, you can save a single frame to a file, like this:
        // frame.save_as_image("frame.png", ImageFormat::Png)?;
        // Or get the raw data like this so you have full control:
        // let data = frame.buffer()?;

        Ok(())
    }

    // Optional handler called when the capture item (usually a window) closes.
    fn on_closed(&mut self) -> Result<(), Self::Error> {
        warn!("Capture window has been closed");

        Ok(())
    }
}

pub fn record_screen(
    recording: Arc<Mutex<bool>>,
    recording_raw: Arc<Mutex<bool>>,
    filename: String,
    capture_config: CaptureConfig,
) -> Result<(), anyhow::Error> {
    // Gets the foreground window, refer to the docs for other capture items
    let primary_monitor = Monitor::primary()?;

    let dimensions = CustomFlags {
        width: primary_monitor.width()?,
        height: primary_monitor.height()?,
        filename: Arc::new(filename),
        capture_config,
        recording_raw: recording_raw.clone(),
    };

    let settings = Settings::new(
        // Item to capture
        primary_monitor,
        // Capture cursor settings
        CursorCaptureSettings::WithCursor,
        // Draw border settings
        DrawBorderSettings::WithoutBorder,

        windows_capture::settings::SecondaryWindowSettings::Default,

        windows_capture::settings::MinimumUpdateIntervalSettings::Default,

        windows_capture::settings::DirtyRegionSettings::Default,
        // The desired color format for the captured frame.
        ColorFormat::Rgba8,
        // Additional flags for the capture settings that will be passed to user defined `new` function.
        dimensions,
    );

    *recording.clone().lock().unwrap() = true;

    // Starts the capture and takes control of the current thread.
    // The errors from handler trait will end up here
    thread::spawn(move || {
        let capture = Capture::start_free_threaded(settings).ok();
        let start = Instant::now();

        loop {
            if !*recording.lock().unwrap()
                || (capture.is_some() && capture.as_ref().unwrap().is_finished())
            {
                println!();
                break;
            }
            print!("\rRecording for {} seconds...", start.elapsed().as_secs());
            let _ = io::stdout().flush();
            thread::sleep(Duration::from_millis(100));
        }

        *recording.lock().unwrap() = false;

        if capture.is_some() {
            capture.unwrap().stop()?;
        }

        *recording_raw.lock().unwrap() = false;

        info!(
            "Capture is done, ran for {} seconds",
            start.elapsed().as_secs()
        );

        Ok(())
    });

    Ok(())
}
