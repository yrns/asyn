use fundsp::prelude::{AttoHash, AudioUnit32, Net32};
use funutd::Rnd;

use crate::types::{Amplitude, Filters, Pitch, Tone, Waveform};

pub fn explosion(seed: u64) -> (Net32, f32) {
    let mut rng = Rnd::from_u64(seed);

    let tone = Tone {
        waveform: Waveform::pick(Waveform::White | Waveform::Pink | Waveform::Brown, &mut rng),
        interpolate_noise: rng.bool(0.5),
        ..Default::default()
    };

    let mut amplitude = Amplitude {
        sustain: rng.f32_in(0.05, 0.1),
        punch: match rng.bool(0.5) {
            true => rng.f32(),
            false => 0.0,
        },
        decay: rng.f32_in(0.3, 0.5),
        ..Default::default()
    };

    if rng.bool(0.5) {
        amplitude.tremolo_depth = rng.f32_in(0.0, 50.0);
        amplitude.tremolo_frequency = rng.f32_in(0.0, 100.0);
    }

    let pitch = Pitch {
        frequency: match tone.waveform {
            Waveform::Brown => rng.f32_in(10_000.0, 20_000.0),
            _ => rng.f32_in(1_000.0, 10_000.0),
        },
        frequency_sweep: rng.f32_in(-1000.0, -5000.0),
        frequency_delta_sweep: rng.f32_in(-1000.0, -5000.0),
        ..Default::default()
    };

    let len = amplitude.len();
    let len1 = 1.0 / len;

    let mut explosion = pitch.to_net(len1) >> (tone.to_net(len1) * amplitude.to_net());

    let mut f = Filters::default();

    if rng.bool(0.5) {
        f.flanger_offset = rng.f32_in(0.0, 10.0);
        f.flanger_offset_sweep = rng.f32_in(-10.0, 10.0);
    }

    if rng.bool(0.5) {
        f.compression = rng.f32_in(0.5, 2.0);
    }

    explosion = explosion >> f.to_net(len1);

    // Make this sound reproducible from the seed.
    explosion.ping(false, AttoHash::new(seed));

    (explosion, len)
}
