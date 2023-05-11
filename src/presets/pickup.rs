use fundsp::prelude::{AttoHash, AudioUnit32, Net32};
use funutd::Rnd;

use crate::{types::Amplitude, Filters, Pitch, Tone, Waveform};

pub fn pickup(seed: u64) -> (Net32, f32) {
    use Waveform::*;

    let mut rng = Rnd::from_u64(seed);

    let tone = Tone::pick(Sine | Square | Whistle | Breaker, &mut rng);

    let amplitude = Amplitude {
        sustain: rng.f32_in(0.02, 0.1),
        punch: rng
            .bool(0.5)
            .then(|| rng.f32_in(0.0, 100.0))
            .unwrap_or_default(),
        decay: rng.f32_in(0.05, 0.4),
        ..Default::default()
    };

    let pitch = Pitch {
        frequency: rng.f32_in(100.0, 2_000.0),
        frequency_jump1: rng
            .bool(0.7)
            .then(|| (rng.f32_in(0.1, 0.3), rng.f32_in(0.1, 1.0)))
            .unwrap_or_default(),
        frequency_jump2: rng
            .bool(0.3)
            .then(|| (rng.f32_in(0.2, 0.4), rng.f32_in(0.1, 1.0)))
            .unwrap_or_default(),
        ..Default::default()
    };

    let f = rng.bool(0.5).then_some(Filters {
        flanger_offset: rng.f32_in(0.0, 10.0),
        flanger_offset_sweep: rng.f32_in(-10.0, 10.0),
        ..Default::default()
    });

    println!(
        "pickup: seed: {} [{}] [{}] [{}]",
        seed, &pitch, &tone, &amplitude
    );

    let len = amplitude.len();
    let len1 = 1.0 / len;

    let mut pickup = pitch.to_net(len1) >> (tone.to_net(len1) * amplitude.to_net());
    if let Some(f) = f {
        pickup = pickup >> f.to_net(len1);
    }
    pickup.ping(false, AttoHash::new(seed));

    (pickup, len)
}
