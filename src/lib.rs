use flagset::{flags, FlagSet};
use fundsp::hacker32::*;
use funutd::Rnd;

pub mod osc;
pub mod play;

#[derive(Copy, Clone, Default)]
pub struct Pitch {
    pub frequency: f32,
    pub frequency_sweep: f32,
    pub frequency_delta_sweep: f32,
    pub vibrato_depth: f32,
    pub vibrato_frequency: f32,
    pub repeat_frequency: f32,
    pub frequency_jump1: (f32, f32),
    pub frequency_jump2: (f32, f32),
}

impl Pitch {
    pub fn to_net(self, len1: f32) -> Net32 {
        wrap(lfo(move |t| {
            self.frequency + self.frequency_sweep * t * len1
        }))
    }
}

flags! {
    #[derive(Default)]
    pub enum Waveform: u32 {
        #[default]
        Sine,
        Triangle,
        Saw,
        Square,
        Tangent,
        Whistle,
        Breaker,
        White,
        Pink,
        Brown,
    }
}

impl Waveform {
    /// Pick a random waveform from the specified set.
    pub fn pick(set: FlagSet<Waveform>, rng: &mut Rnd) -> Self {
        let i = rng.u32_to(set.into_iter().count() as u32) as usize;
        set.into_iter().nth(i).unwrap()
    }
}

#[derive(Clone, Default)]
pub struct Tone {
    pub waveform: Waveform,
    pub interpolate_noise: bool,
    pub square_duty: f32,
    pub square_duty_sweep: f32,
    pub harmonics: u32,
    pub harmonics_falloff: f32,
}

impl Tone {
    pub fn pick(set: FlagSet<Waveform>, rng: &mut Rnd) -> Self {
        Self {
            waveform: Waveform::pick(set, rng),
            square_duty: if set.contains(Waveform::Square) {
                rng.f32_in(0.0, 100.0)
            } else {
                0.0
            },
            square_duty_sweep: if set.contains(Waveform::Square) {
                rng.f32_in(-100.0, 100.0)
            } else {
                0.0
            },
            ..Default::default()
        }
    }

    pub fn to_net(self, len1: f32) -> Net32 {
        match self.waveform {
            Waveform::Sine => wrap(sine()),
            Waveform::Triangle => wrap(osc::triangle()),
            Waveform::Saw => wrap(osc::saw()),
            Waveform::Square => {
                // Duty sweep should repeat w/ the frequency repeat...
                let duty = wrap(lfo(move |t| {
                    lerp(
                        0.01,
                        0.99,
                        self.square_duty + self.square_duty_sweep * t * len1,
                    )
                }));
                (pass() | duty) >> osc::square()
            }
            Waveform::Tangent => wrap(osc::tangent()),
            Waveform::Whistle => wrap(osc::whistle()),
            Waveform::Breaker => wrap(osc::breaker()),
            Waveform::White => wrap(osc::white(self.interpolate_noise)),
            Waveform::Pink => wrap(osc::white(self.interpolate_noise) >> pinkpass()),
            Waveform::Brown => {
                wrap(osc::white(self.interpolate_noise) >> lowpole_hz(10.0) * dc(13.7))
            }
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Amplitude {
    pub attack: f32,
    pub sustain: f32,
    pub punch: f32,
    pub decay: f32,
    pub tremolo_depth: f32,
    pub tremolo_frequency: f32,
}

impl Amplitude {
    pub fn len(&self) -> f32 {
        self.attack + self.sustain + self.decay
    }

    pub fn to_net(self) -> Net32 {
        let mut a = wrap(lfo(move |t| aspd(self, t)));
        if self.tremolo_depth > 0.0 {
            a = a * tremolo(self.tremolo_depth, self.tremolo_frequency);
        }
        a
    }
}

pub fn aspd(amplitude: Amplitude, t: f32) -> f32 {
    let Amplitude {
        attack,
        sustain,
        punch,
        decay,
        ..
    } = amplitude;

    if t < attack {
        lerp(0.0, 1.0 - punch, t / attack)
    } else if t < (attack + sustain) {
        if punch > 0.0 {
            lerp(1.0, 1.0 - punch, (t - attack) / sustain)
        } else {
            1.0
        }
    } else {
        clamp01(lerp(1.0 - punch, 0.0, (t - attack - sustain) / decay))
    }
}

#[derive(Clone)]
pub struct Filters {
    pub flanger_offset: f32,
    pub flanger_offset_sweep: f32,
    pub bit_crush: f32,
    pub bit_crush_sweep: f32,
    pub low_pass_cutoff: f32,
    pub low_pass_sweep: f32,
    pub high_pass_cutoff: f32,
    pub high_pass_sweep: f32,
    pub compression: f32,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            flanger_offset: 0.0,
            flanger_offset_sweep: 0.0,
            bit_crush: 0.0,
            bit_crush_sweep: 0.0,
            low_pass_cutoff: 22_050.0,
            low_pass_sweep: 0.0,
            high_pass_cutoff: 0.0,
            high_pass_sweep: 0.0,
            compression: 1.0,
        }
    }
}

impl Filters {
    pub fn to_net(self, len1: f32) -> Net32 {
        let mut f = wrap(pass());

        // Make feedback a parameter?
        if self.flanger_offset > 0.0 {
            f = f
                >> flanger(0.0, 0.0, 0.1, move |t| {
                    self.flanger_offset + self.flanger_offset_sweep * t * len1
                });
        }

        if self.low_pass_cutoff < 22_050.0 {
            f = (f | lfo(move |t| self.low_pass_cutoff + self.low_pass_sweep * t * len1))
                >> lowpole();
        }

        if self.high_pass_cutoff > 0.0 {
            f = (f | lfo(move |t| self.high_pass_cutoff + self.high_pass_sweep * t * len1))
                >> highpole();
        }

        let c = dbg!(self.compression);
        if c != 1.0 {
            f = f
                >> map(move |f: &Frame<f32, U1>| {
                    let sample = f[0];
                    if sample >= 0.0 {
                        sample.pow(c)
                    } else {
                        -((-sample).pow(c))
                    }
                });
        }

        f
    }
}

pub fn cosine() -> An<Sine<f32>> {
    An(Sine::with_phase(DEFAULT_SR, Some(0.25f32)))
}

pub fn tremolo(
    depth: f32,
    frequency: f32,
) -> An<impl AudioNode<Sample = f32, Inputs = U0, Outputs = U1>> {
    1.0 - (depth * (0.5 + 0.5 * (constant(frequency) >> cosine())))
}

pub fn wrap(unit: impl AudioUnit32 + 'static) -> Net32 {
    let unit: Box<dyn AudioUnit32> = Box::new(unit);
    Net32::wrap(unit)
}

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

pub fn explosion(seed: u64) -> (Net32, f32) {
    let mut rng = Rnd::from_u64(seed);

    let tone = Tone {
        waveform: Waveform::pick(Waveform::White | Waveform::Pink | Waveform::Brown, &mut rng),
        interpolate_noise: rng.bool(0.5),
        ..Default::default()
    };

    let mut amplitude = Amplitude {
        sustain: rng.f32_in(0.05, 0.1),
        punch: match rng.bool(0.5) {
            true => rng.f32(),
            false => 0.0,
        },
        decay: rng.f32_in(0.3, 0.5),
        ..Default::default()
    };

    if rng.bool(0.5) {
        amplitude.tremolo_depth = rng.f32_in(0.0, 50.0);
        amplitude.tremolo_frequency = rng.f32_in(0.0, 100.0);
    }

    let pitch = Pitch {
        frequency: match tone.waveform {
            Waveform::Brown => rng.f32_in(10_000.0, 20_000.0),
            _ => rng.f32_in(1_000.0, 10_000.0),
        },
        frequency_sweep: rng.f32_in(-1000.0, -5000.0),
        frequency_delta_sweep: rng.f32_in(-1000.0, -5000.0),
        ..Default::default()
    };

    let len = amplitude.len();
    let len1 = 1.0 / len;

    let mut explosion = pitch.to_net(len1) >> (tone.to_net(len1) * amplitude.to_net());

    let mut f = Filters::default();

    if rng.bool(0.5) {
        f.flanger_offset = rng.f32_in(0.0, 10.0);
        f.flanger_offset_sweep = rng.f32_in(-10.0, 10.0);
    }

    if rng.bool(0.5) {
        f.compression = rng.f32_in(0.5, 2.0);
    }

    explosion = explosion >> f.to_net(len1);

    // Make this sound reproducible from the seed.
    explosion.ping(false, AttoHash::new(seed));

    (wrap(explosion), len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let len = 0.25;
        //let mut jump = (constant(22.0) | constant(0.5)) >> harmonic(osc::square(), 3, 0.5);

        //let mut jump = sine_hz(110.0) >> map(|i: &Frame<f32, U1>| dbg!(i[0]));
        let (mut jump, len) = explosion(0);

        //let len = 0.1;
        //let mut jump = dc(220.0 / DEFAULT_SR as f32) >> resample(white());
        //let mut jump = dc(44.1) >> osc::white(false); // >> resonator_hz(0.0, 220.0);

        println!("{}", jump.display());

        let wav = Wave32::render(44100.0, len as f64, &mut jump);
        dbg!(wav.amplitude());
        //wav.normalize();
        //dbg!(wav.amplitude());

        //wav.write_wav16(&mut std::io::stdout().lock()).unwrap();
        let path = std::path::Path::new("jump.wav");
        wav.save_wav16(path).unwrap();
    }
}
