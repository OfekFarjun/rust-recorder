// For future use, when we have the vod-hub planned to happen.

use std::time::Duration;

use reqwest::{header::{HeaderMap, HeaderValue, CONTENT_TYPE}, StatusCode};
use reqwest::{multipart, Client, Body};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

pub fn uploader_task(recordings_folder: &String) {
    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(1 * 60)).await;
            upload_local_files().await
        }
    });
}

async fn upload_local_files() {

}

async fn remove_file() {

}

async fn upload_file(file_name: String) -> Result<bool, Box<dyn std::error::Error>> {
    let client = Client::new();
    let file = File::open(file_name.clone()).await?;
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_body = Body::wrap_stream(stream);

    let file_part = multipart::Part::stream(file_body)
        .file_name(file_name)
        .mime_str("video/mp4")?;

        let form = reqwest::multipart::Form::new()
        .text("stationId", "123")
        .part("file", file_part);

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("multipart/form-data"),
    );

    let res = client
        .post("http://localhost:3000/files")
        .multipart(form)
        .send()
        .await?;

    info!("{}", res.status());
    Ok(res.status() == StatusCode::OK)
}