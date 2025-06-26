use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
    thread, time,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample,
};
use log::{error, info};

pub fn record_audio(
    recording: Arc<Mutex<bool>>,
    recording_raw: Arc<Mutex<bool>>,
    filename: String,
) -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(anyhow::Error::msg("Could not find default input device"))?;

    let config = device.default_input_config()?;

    let spec = hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: if config.sample_format().is_float() {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    };

    let writer = hound::WavWriter::create(filename, spec)?;

    let writer = Arc::new(Mutex::new(writer));

    let err_fn = move |err| {
        error!("An error occurred on audio stream: {}", err);
    };

    let writer_ptr = writer.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_ptr),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_ptr),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_ptr),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_ptr),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    info!(
        "Starting audio recording on device: {}",
        device.name().unwrap_or("None".into())
    );
    stream.play()?;

    thread::spawn(move || {
        loop {
            if !*recording.lock().unwrap() {
                break;
            }
            thread::sleep(time::Duration::from_millis(100));
        }
        *recording.lock().unwrap() = false;
        drop(stream);
        *recording_raw.lock().unwrap() = false;
        info!("Audio recording stopped");
    });

    Ok(())
}

type WavWriterHandle = Arc<Mutex<hound::WavWriter<BufWriter<File>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut writer) = writer.try_lock() {
        for &sample in input.iter() {
            let sample: U = U::from_sample(sample);
            writer.write_sample(sample).ok();
        }
    }
}
