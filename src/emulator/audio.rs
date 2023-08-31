use std::sync::{Arc, Mutex};

use cpal::{FromSample, Sample, SampleFormat, Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioPlayer {
    pub buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    pub sample_rate: u32,
}

pub fn setup_audio_device() -> (AudioPlayer, Stream) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    let wanted_sample_rate = cpal::SampleRate(44100);

    let mut supported_configs = device.supported_output_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs.find_map(|f| {
        if f.channels() == 2 && f.sample_format() == cpal::SampleFormat::F32 {
            if f.min_sample_rate() <= wanted_sample_rate && wanted_sample_rate <= f.max_sample_rate() {
                Some(f.with_sample_rate(wanted_sample_rate))
            } else {
                Some(f.with_max_sample_rate())
            }
        } else {
            None
        }
    }).expect("Found no config");


    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();

    let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);


    let shared_buffer = Arc::new(Mutex::new(Vec::new()));
    let stream_buffer = shared_buffer.clone();


    let player = AudioPlayer {
        buffer: shared_buffer,
        sample_rate: config.sample_rate.0,
    };

    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(&config, move |data: &mut [f32], _| cpal_thread(data, &stream_buffer), err_fn, None),
        SampleFormat::I16 => device.build_output_stream(&config, move |data: &mut [i16], _| cpal_thread(data, &stream_buffer), err_fn, None),
        SampleFormat::U16 => device.build_output_stream(&config, move |data: &mut [u16], _| cpal_thread(data, &stream_buffer), err_fn, None),
        sample_format => unreachable!("Unhandled sample format! {}", sample_format),
    }.unwrap();

    stream.play().unwrap();

    (player, stream)
}

fn cpal_thread<T: FromSample<f32>>(outbuffer: &mut [T], audio_buffer: &Arc<Mutex<Vec<(f32, f32)>>>) {
    let mut inbuffer = audio_buffer.lock().unwrap();
    let outlen = ::std::cmp::min(outbuffer.len() / 2, inbuffer.len());
    for (i, (in_l, in_r)) in inbuffer.drain(..outlen).enumerate() {
        outbuffer[i * 2] = (&in_l).to_sample();
        outbuffer[i * 2 + 1] = (&in_r).to_sample();
    }
    if inbuffer.len() > 2048 {
        inbuffer.truncate(512)
    }
}
