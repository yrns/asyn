use crate::{types::Amplitude, Asyn, Pitch, Tone, Waveform};

pub fn blip(seed: u64) -> Asyn {
    use Waveform::*;

    let mut rng = funutd::Rnd::from_u64(seed);

    Asyn {
        seed,
        tone: {
            let mut tone = Tone {
                waveform: Waveform::pick(
                    Sine | Triangle | Saw | Square | Tangent | Whistle | Breaker,
                    &mut rng,
                ),
                square_duty: rng.f32_in(0.1, 0.9),
                ..Default::default()
            };
            if rng.bool(0.5) {
                tone.harmonics = rng.u32_in(1, 5);
                tone.harmonics_falloff = rng.f32();
            }
            tone
        },
        amplitude: Amplitude {
            sustain: rng.f32_in(0.01, 0.07),
            decay: rng.f32_in(0.0, 0.03),
            ..Default::default()
        },
        pitch: Pitch {
            frequency: rng.f32_in(100.0, 3_000.0),
            ..Default::default()
        },

        ..Default::default()
    }
}
