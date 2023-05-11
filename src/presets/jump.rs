use fundsp::prelude::{AttoHash, AudioUnit32, Net32};
use funutd::Rnd;

use crate::types::{Amplitude, Filters, Pitch, Tone, Waveform};

pub fn jump(seed: u64) -> (Net32, f32) {
    let mut rng = Rnd::from_u64(seed);

    let amplitude = Amplitude {
        sustain: rng.f32_in(0.02, 0.1),
        decay: rng.f32_in(0.05, 0.4),
        punch: match rng.bool(0.5) {
            true => rng.f32(),
            false => 0.0,
        },
        ..Default::default()
    };

    let len = amplitude.len();
    let len1 = 1.0 / len;

    let pitch = Pitch {
        frequency: rng.f32_in(100.0, 2000.0),
        frequency_sweep: rng.f32_in(200.0, 2000.0),
        ..Default::default()
    }
    .to_net(len1);

    let tone = Tone::pick(
        Waveform::Sine | Waveform::Square | Waveform::Whistle | Waveform::Breaker,
        &mut rng,
    );

    let mut f = Filters::default();

    // Flanger.
    if rng.bool(0.3) {
        f.flanger_offset = rng.f32_in(0.0, 10.0);
        f.flanger_offset_sweep = rng.f32_in(-10.0, 10.0);
    }

    // Low pass filter.
    if rng.bool(0.5) {
        f.low_pass_cutoff = rng.f32_in(0.0, 22050.0);
        f.low_pass_sweep = rng.f32_in(-22050.0, 22050.0);
    }

    // High pass filter.
    if rng.bool(0.5) {
        f.high_pass_cutoff = rng.f32_in(0.0, 22050.0);
        f.high_pass_sweep = rng.f32_in(-22050.0, 22050.0);
    }

    let mut jump = pitch >> tone.to_net(len1) * amplitude.to_net() >> f.to_net(len1);
    jump.ping(false, AttoHash::new(seed));

    (jump, len)
}
