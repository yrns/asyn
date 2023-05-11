use crate::types::{Amplitude, Asyn, Filters, Pitch, Tone, Waveform};

pub fn jump(seed: u64) -> Asyn {
    let mut rng = funutd::Rnd::from_u64(seed);

    let mut filters = Filters::default();

    // Flanger.
    if rng.bool(0.3) {
        filters.flanger_offset = rng.f32_in(0.0, 10.0);
        filters.flanger_offset_sweep = rng.f32_in(-10.0, 10.0);
    }

    // Low pass filter.
    if rng.bool(0.5) {
        filters.low_pass_cutoff = rng.f32_in(0.0, 22050.0);
        filters.low_pass_sweep = rng.f32_in(-22050.0, 22050.0);
    }

    // High pass filter.
    if rng.bool(0.5) {
        filters.high_pass_cutoff = rng.f32_in(0.0, 22050.0);
        filters.high_pass_sweep = rng.f32_in(-22050.0, 22050.0);
    }

    Asyn {
        amplitude: Amplitude {
            sustain: rng.f32_in(0.02, 0.1),
            decay: rng.f32_in(0.05, 0.4),
            punch: match rng.bool(0.5) {
                true => rng.f32(),
                false => 0.0,
            },
            ..Default::default()
        },

        pitch: Pitch {
            frequency: rng.f32_in(100.0, 2000.0),
            frequency_sweep: rng.f32_in(200.0, 2000.0),
            ..Default::default()
        },

        tone: Tone::pick(
            Waveform::Sine | Waveform::Square | Waveform::Whistle | Waveform::Breaker,
            &mut rng,
        ),
        filters: Some(filters),
        ..Default::default()
    }
}
