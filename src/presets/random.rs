use crate::types::{Amplitude, Asyn, Filters, Pitch, Tone, Waveform};

pub fn random(rng: &mut funutd::Rnd) -> Asyn {
    // This is pretty obtuse.
    let asd = rng.u32_in(3, 16);

    let attack = if (asd & 1) > 0 && (asd & 2) > 0 {
        rng.f32_in(0.0, 2.0)
    } else {
        0.0
    };

    let (sustain, punch) = if (asd & 4) > 0 {
        (rng.f32(), if rng.bool(0.5) { rng.f32() } else { 0.0 })
    } else {
        (0.0, 0.0)
    };

    let decay = if (asd & 8) > 0 {
        rng.f32_in(0.0, 5.0)
    } else {
        0.0
    };

    let (tremolo_depth, tremolo_frequency) = if rng.bool(0.5) {
        (rng.f32(), rng.f32_in(0.0, 1000.0))
    } else {
        (0.0, 0.0)
    };

    let amplitude = Amplitude {
        attack,
        sustain,
        punch,
        decay,
        tremolo_depth,
        tremolo_frequency,
    };

    let mut f = Filters::default();

    if rng.bool(0.5) {
        f.flanger_offset = rng.f32_in(0.0, 50.0);
        if rng.bool(0.5) {
            f.flanger_offset_sweep = rng.f32_in(-50.0, 50.0);
        }
    }

    if rng.bool(0.2) {
        f.bit_crush = rng.i32_in(1, 16);
        if rng.bool(0.5) {
            f.bit_crush_sweep = rng.i32_in(-16, 16);
        }
    }

    // jfxr has a typo/bug with this, so we can't compare.
    // TODO: this can easily generate an empty signal if we do both
    if rng.bool(0.5) {
        f.low_pass_cutoff = rng.f32_in(0.0, 10_000.0);
        if rng.bool(0.5) {
            f.low_pass_sweep = rng.f32_in(-22_050.0, 22_050.0);
        }
    } else if rng.bool(0.5) {
        f.high_pass_cutoff = rng.f32_in(0.0, 10_000.0);
        if rng.bool(0.5) {
            f.high_pass_sweep = rng.f32_in(-22_050.0, 22_050.0);
        }
    }

    if rng.bool(0.5) {
        f.compression = rng.f32_in(0.5, 2.0);
    }

    // TODO: normalization/amplification

    let mut pitch = Pitch {
        frequency: rng.f32_in(10.0, 10_000.0),
        frequency_sweep: if rng.bool(0.5) {
            rng.f32_in(-10_000.0, 10_000.0)
        } else {
            0.0
        },
        frequency_delta_sweep: if rng.bool(0.5) {
            rng.f32_in(-10_000.0, 10_000.0)
        } else {
            0.0
        },
        ..Default::default()
    };

    let repeat = rng.u32_in(0, 2);

    if repeat >= 1 {
        pitch.repeat_frequency = rng.f32_in(1.0 / amplitude.len(), 100.0);
    }

    if repeat >= 2 {
        pitch.frequency_jump1 = (rng.f32(), rng.f32_in(-100.0, 100.0));
        if rng.bool(0.5) {
            pitch.frequency_jump1 = (rng.f32(), rng.f32_in(-100.0, 100.0));
            if pitch.frequency_jump1.0 < pitch.frequency_jump2.0 {
                std::mem::swap(&mut pitch.frequency_jump1.0, &mut pitch.frequency_jump2.0);
            }
        }
    }

    if rng.bool(0.5) {
        pitch.vibrato_depth = rng.f32_in(0.0, 1_000.0);
        pitch.vibrato_frequency = rng.f32_in(0.0, 1_000.0);
    }

    let mut tone = Tone::pick(flagset::FlagSet::<Waveform>::full(), rng);

    if tone.interpolate_noise {
        tone.interpolate_noise = rng.bool(0.5);
    }

    if rng.bool(0.5) {
        tone.harmonics = rng.u32_in(0, 5);
        tone.harmonics_falloff = rng.f32();
    }

    Asyn {
        seed: rng.stream(),
        pitch,
        tone,
        amplitude,
        filters: Some(f),
        ..Default::default()
    }
}
