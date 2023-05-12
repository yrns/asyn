use crate::types::{Amplitude, Asyn, Pitch, Tone, Waveform};

pub fn powerup(seed: u64) -> Asyn {
    use Waveform::*;

    let mut rng = funutd::Rnd::from_u64(seed);

    let mut powerup = Asyn {
        tone: Tone::pick(
            Sine | Triangle | Saw | Square | Tangent | Whistle | Breaker,
            &mut rng,
        ),
        amplitude: Amplitude {
            sustain: rng.f32_in(0.05, 0.2),
            punch: rng.bool(0.5).then(|| rng.f32()).unwrap_or_default(),
            decay: rng.f32_in(0.1, 0.4),
            ..Default::default()
        },
        pitch: Pitch {
            frequency: rng.f32_in(500.0, 2_000.0),
            frequency_sweep: rng.f32_in(0.0, 2_000.0),
            frequency_delta_sweep: rng.f32_in(0.0, 2_000.0),
            repeat_frequency: rng
                .bool(0.5)
                .then(|| rng.f32_in(0.0, 20.0))
                .unwrap_or_default(),
            ..Default::default()
        },
        ..Default::default()
    };

    if rng.bool(0.5) {
        powerup.pitch.vibrato_depth = rng.f32_in(0.0, 1000.0);
        powerup.pitch.vibrato_frequency = rng.f32_in(0.0, 1000.0);
    }

    powerup
}
