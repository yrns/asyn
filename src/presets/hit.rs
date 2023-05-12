use crate::types::{Amplitude, Asyn, Filters, Pitch, Tone, Waveform};

pub fn hit(seed: u64) -> Asyn {
    use Waveform::*;

    let mut rng = funutd::Rnd::from_u64(seed);

    let mut f = Filters {
        low_pass_sweep: rng.f32_in(-22_050.0, 22_050.0),
        ..Default::default()
    };

    if rng.bool(0.5) {
        f.flanger_offset = rng.f32_in(0.0, 10.0);
        f.flanger_offset_sweep = rng.f32_in(-10.0, 10.0);
    }

    Asyn {
        seed,
        pitch: Pitch {
            frequency: rng.f32_in(500.0, 1_000.0),
            frequency_sweep: rng.f32_in(-200.0, -1_000.0),
            frequency_delta_sweep: rng.f32_in(-200.0, -1_000.0),
            ..Default::default()
        },
        tone: Tone::from(Waveform::pick(
            Saw | Square | Tangent | White | Pink | Brown,
            &mut rng,
        )),
        amplitude: Amplitude {
            sustain: rng.f32_in(0.02, 0.1),
            punch: rng.bool(0.5).then(|| rng.f32()).unwrap_or_default(),
            decay: rng.f32_in(0.02, 0.1),
            ..Default::default()
        },
        filters: Some(f),
    }
}
