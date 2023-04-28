use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker32::*;

pub fn play(c: impl AudioUnit32 + 'static) {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(c, &device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(c, &device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(c, &device, &config.into()),
        _ => panic!("Unsupported format"),
    }
}

pub fn run<T>(mut c: impl AudioUnit32 + 'static, device: &cpal::Device, config: &cpal::StreamConfig)
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    c.reset(Some(sample_rate));
    c.allocate();

    let mut next_value = move || c.get_stereo();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let sample = next_value();
                    let left = T::from_sample(sample.0);
                    let right: T = T::from_sample(sample.1);

                    for (channel, sample) in frame.iter_mut().enumerate() {
                        if channel & 1 == 0 {
                            *sample = left;
                        } else {
                            *sample = right;
                        }
                    }
                }
            },
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(333));
}
