use std::process::{Command};

use anyhow::Error;
use log::{info, warn};

pub fn combine_outputs(filename: &str) -> Result<(), anyhow::Error> {
    let video = format!("{}.mp4", filename);
    let audio = format!("{}.wav", filename);

    if !std::fs::exists(&audio).is_ok_and(|exists| exists) {
        warn!("Not combining, there is no audio file present");
        return Ok(())
    }

    let output = format!("{}-combined.mp4", filename);
    let mut out = Command::new("cmd")
        .args([
            "/C",
            &format!("ffmpeg.exe -i {video} -i {audio} -c copy {output}"),
        ])
        .spawn()
        .or(Err(Error::msg("Could not combine files")))?;

    let code = out.wait()?;

    if !code.success() {
        return Err(Error::msg("Could not combine outputs via ffmpeg.exe"));
    }

    std::fs::remove_file(video)?;
    std::fs::remove_file(audio)?;

    info!("Done combining via ffmpeg.exe");

    Ok(())
}
