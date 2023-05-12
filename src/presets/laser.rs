use crate::{types::Amplitude, Asyn, Filters, Pitch, Tone, Waveform};

pub fn laser(seed: u64) -> Asyn {
    use Waveform::*;

    let mut rng = funutd::Rnd::from_u64(seed);

    Asyn {
        seed,
        tone: Tone::pick(
            Sine | Triangle | Saw | Square | Tangent | Whistle | Breaker,
            &mut rng,
        ),
        amplitude: Amplitude {
            sustain: rng.f32_in(0.02, 0.1),
            punch: rng.bool(0.5).then(|| rng.f32()).unwrap_or_default(),
            decay: rng.f32_in(0.02, 0.1),
            ..Default::default()
        },
        pitch: {
            let mut pitch = Pitch {
                frequency: rng.f32_in(500.0, 2_000.0),
                frequency_sweep: rng.f32_in(-200.0, -2_000.0),
                frequency_delta_sweep: rng.f32_in(-200.0, -2_000.0),
                ..Default::default()
            };
            if rng.bool(0.5) {
                pitch.vibrato_depth = rng.f32_in(0.0, 0.5 * pitch.frequency);
                pitch.vibrato_frequency = rng.f32_in(0.0, 100.0);
            }
            pitch
        },
        filters: rng.bool(0.5).then_some(Filters {
            flanger_offset: rng.f32_in(0.0, 10.0),
            flanger_offset_sweep: rng.f32_in(-10.0, 10.0),
            ..Default::default()
        }),
        ..Default::default()
    }
}
