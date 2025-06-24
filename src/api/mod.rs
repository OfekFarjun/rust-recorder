use axum::{
    extract::State,
    http::StatusCode,
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

use crate::{capture, config::Config, keep_alive::keep_alive_task};

#[derive(Deserialize, Serialize)]
pub struct Status {
    pub message: String,
    pub recording: bool,
    pub last_keep_alive: u64,
}

pub struct AppState {
    pub recording: Arc<Mutex<bool>>,
    pub recording_raw: Arc<Mutex<bool>>,
    pub last_keep_alive: Mutex<u64>,
    pub config: Config,
}

pub async fn start(config: Config) {
    let shared_state = Arc::new(AppState {
        recording: Arc::new(Mutex::new(false)),
        recording_raw: Arc::new(Mutex::new(false)),
        last_keep_alive: Mutex::new(0),
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

async fn status(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!(&Status {
        message: "Recorder is ok!".to_string(),
        recording: *state.recording.lock().unwrap() || *state.recording_raw.lock().unwrap(),
        last_keep_alive: *state.last_keep_alive.lock().unwrap()
    })))
}

async fn start_recording(State(state): State<Arc<AppState>>) -> Result<String, StatusCode> {
    let recording: bool = *state.recording.lock().unwrap();
    let recording_raw: bool = *state.recording_raw.lock().unwrap();
    match recording || recording_raw {
        false => {
            capture::record_screen(
                state.recording.clone(),
                state.recording_raw.clone(),
                format!(
                    "{}/{}",
                    state.config.recordings_folder,
                    Local::now().format("%d.%m.%Y-%H_%M_%S.mp4").to_string()
                ),
                state.config.capture,
            );
            refresh_keep_alive(state);
            Ok(format!("Screen capture started"))
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

async fn stop_recording(State(state): State<Arc<AppState>>) -> Result<String, StatusCode> {
    let recording: bool = *state.recording.lock().unwrap();
    let recording_raw: bool = *state.recording_raw.lock().unwrap();
    match recording || recording_raw {
        true => {
            *state.recording.lock().unwrap() = false;
            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;
                if !*state.recording_raw.lock().unwrap() {
                    break;
                }
            }

            Ok(format!("Screen capture stopped"))
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

async fn keep_alive(State(state): State<Arc<AppState>>) -> Result<String, StatusCode> {
    refresh_keep_alive(state);

    Ok(format!("Ok"))
}

fn refresh_keep_alive(state: Arc<AppState>) {
    *state.last_keep_alive.lock().unwrap() = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
}
