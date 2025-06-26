pub mod errors;

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
    u64,
};

use crate::{
    api::errors::ApiError, audio, capture, config::Config, ffmpeg, keep_alive::keep_alive_task,
};

#[derive(Deserialize, Serialize)]
pub struct Status {
    pub message: String,
    pub recording: bool,
    pub last_keep_alive: u64,
}

pub struct AppState {
    pub recording: Arc<Mutex<bool>>,
    pub recording_screen_raw: Arc<Mutex<bool>>,
    pub recording_audio_raw: Arc<Mutex<bool>>,
    pub last_keep_alive: Mutex<u64>,
    pub filename: Mutex<String>,
    pub config: Config,
}

pub async fn start(config: Config) {
    let shared_state = Arc::new(AppState {
        recording: Arc::new(Mutex::new(false)),
        recording_screen_raw: Arc::new(Mutex::new(false)),
        recording_audio_raw: Arc::new(Mutex::new(false)),
        last_keep_alive: Mutex::new(0),
        filename: Mutex::new(format!("None")),
        config,
    });

    // build our application with a single route
    let app = Router::new()
        .route("/status", get(status))
        .route("/start", post(start_recording))
        .route("/stop", post(stop_recording))
        .route("/keep_alive", post(keep_alive))
        .with_state(shared_state.clone());

    keep_alive_task(shared_state.clone());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn status(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!(&Status {
        message: "Recorder is ok!".to_string(),
        recording: *state.recording.lock().unwrap() || *state.recording_screen_raw.lock().unwrap(),
        last_keep_alive: *state.last_keep_alive.lock().unwrap()
    })))
}

async fn start_recording(State(state): State<Arc<AppState>>) -> Result<String, ApiError> {
    let recording: bool = *state.recording.lock().unwrap();
    let recording_screen_raw: bool = *state.recording_screen_raw.lock().unwrap();
    let recording_audio_raw = *state.recording_audio_raw.lock().unwrap();
    match recording || recording_screen_raw || recording_audio_raw {
        false => {
            let filename = format!(
                "{}/{}",
                state.config.recordings_folder,
                Local::now().format("%d.%m.%Y-%H_%M_%S")
            );
            *state.filename.lock().unwrap() = filename.clone();
            capture::record_screen(
                state.recording.clone(),
                state.recording_screen_raw.clone(),
                format!("{filename}.mp4"),
                state.config.capture,
            ).or(Err(ApiError::InternalServerError))?;
            audio::record_audio(
                state.recording.clone(),
                state.recording_audio_raw.clone(),
                format!("{filename}.wav"),
            ).or(Err(ApiError::InternalServerError))?;
            refresh_keep_alive(state);
            Ok(format!("Screen capture started"))
        }
        _ => Err(ApiError::CaptureAlreadyInProgress),
    }
}

async fn stop_recording(State(state): State<Arc<AppState>>) -> Result<String, ApiError> {
    let recording: bool = *state.recording.lock().unwrap();
    let recording_screen_raw: bool = *state.recording_screen_raw.lock().unwrap();
    let recording_audio_raw = *state.recording_audio_raw.lock().unwrap();
    match recording || recording_screen_raw || recording_audio_raw {
        true => {
            *state.recording.lock().unwrap() = false;
            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;
                if !*state.recording_screen_raw.lock().unwrap()
                    && !*state.recording_audio_raw.lock().unwrap()
                {
                    break;
                }
            }

            ffmpeg::combine_outputs(&*state.filename.lock().unwrap())
                .or(Err(ApiError::InternalServerError))?;

            Ok(format!("Capture stopped successfully"))
        }
        _ => Err(ApiError::NoCaptureIsRunning),
    }
}

async fn keep_alive(State(state): State<Arc<AppState>>) -> Result<String, ApiError> {
    refresh_keep_alive(state);

    Ok(format!("Ok"))
}

fn refresh_keep_alive(state: Arc<AppState>) {
    *state.last_keep_alive.lock().unwrap() = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
}
