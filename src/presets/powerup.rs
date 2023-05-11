use fundsp::prelude::{AttoHash, AudioUnit32, Net32};
use funutd::Rnd;

use crate::types::{Amplitude, Pitch, Tone, Waveform};

pub fn powerup(seed: u64) -> (Net32, f32) {
    use Waveform::*;

    let mut rng = Rnd::from_u64(seed);

    let tone = Tone::pick(
        Sine | Triangle | Saw | Square | Tangent | Whistle | Breaker,
        &mut rng,
    );

    let amplitude = Amplitude {
        sustain: rng.f32_in(0.05, 0.2),
        punch: rng
            .bool(0.5)
            .then(|| rng.f32_in(0.0, 100.0))
            .unwrap_or_default(),
        decay: rng.f32_in(0.1, 0.4),
        ..Default::default()
    };

    let mut pitch = Pitch {
        frequency: rng.f32_in(500.0, 2_000.0),
        frequency_sweep: rng.f32_in(0.0, 2_000.0),
        frequency_delta_sweep: rng.f32_in(0.0, 2_000.0),
        repeat_frequency: rng
            .bool(0.5)
            .then(|| rng.f32_in(0.0, 20.0))
            .unwrap_or_default(),
        ..Default::default()
    };

    if rng.bool(0.5) {
        pitch.vibrato_depth = rng.f32_in(0.0, 1000.0);
        pitch.vibrato_frequency = rng.f32_in(0.0, 1000.0);
    }

    println!(
        "powerup: seed: {} [{}] [{}] [{}]",
        seed, &pitch, &tone, &amplitude
    );

    let len = amplitude.len();
    let len1 = 1.0 / len;

    let mut powerup = pitch.to_net(len1) >> (tone.to_net(len1) * amplitude.to_net());
    powerup.ping(false, AttoHash::new(seed));

    (powerup, len)
}
