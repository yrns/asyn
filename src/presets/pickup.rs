use crate::{types::Amplitude, Asyn, Filters, Pitch, Tone, Waveform};

pub fn pickup(seed: u64) -> Asyn {
    use Waveform::*;

    let mut rng = funutd::Rnd::from_u64(seed);

    Asyn {
        seed,
        tone: Tone::pick(Sine | Square | Whistle | Breaker, &mut rng),
        amplitude: Amplitude {
            sustain: rng.f32_in(0.02, 0.1),
            punch: rng.bool(0.5).then(|| rng.f32()).unwrap_or_default(),
            decay: rng.f32_in(0.05, 0.4),
            ..Default::default()
        },
        pitch: Pitch {
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
        },
        filters: rng.bool(0.5).then_some(Filters {
            flanger_offset: rng.f32_in(0.0, 10.0),
            flanger_offset_sweep: rng.f32_in(-10.0, 10.0),
            ..Default::default()
        }),
        ..Default::default()
    }
}
