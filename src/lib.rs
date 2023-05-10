use std::fmt::Display;

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
    // This does nothing without sweep.
    pub repeat_frequency: f32,
    pub frequency_jump1: (f32, f32), // onset %, amount %
    pub frequency_jump2: (f32, f32),
}

impl Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0} hz", self.frequency)?;
        if self.frequency_sweep > 0.0 {
            write!(f, " sweep: {:.0}", self.frequency_sweep)?;
        }
        if self.frequency_delta_sweep > 0.0 {
            write!(f, " delta sweep: {:.0}", self.frequency_sweep)?;
        }
        if self.vibrato_depth > 0.0 && self.vibrato_frequency > 0.0 {
            write!(
                f,
                " vibrato: ({:.0}, {:.0})",
                self.vibrato_depth, self.vibrato_frequency
            )?;
        }
        if self.repeat_frequency > 0.0 {
            write!(f, " repeat: {:.0}", self.repeat_frequency)?;
        }
        if self.frequency_jump1.0 > 0.0 {
            write!(f, " jump1: {:?}", self.frequency_jump1)?;
        }
        if self.frequency_jump2.0 > 0.0 {
            write!(f, " jump1: {:?}", self.frequency_jump2)?;
        }
        Ok(())
    }
}

impl Pitch {
    pub fn to_net(self, len1: f32) -> Net32 {
        wrap(lfo(move |t| {
            // t in repetition.
            let t_repeat = fract(t * len1 * self.repeat_frequency.max(len1));

            let mut f = self.frequency
                + t_repeat * self.frequency_sweep
                // Delta sweep is quadratic.
                + t_repeat * t_repeat * self.frequency_delta_sweep;

            // Jump 1.
            let jump = self.frequency_jump1;
            if t_repeat > jump.0 {
                f *= 1.0 + jump.1;
            }

            // Jump 2.
            let jump = self.frequency_jump2;
            if t_repeat > jump.0 {
                f *= 1.0 + jump.1;
            }

            // Vibrato.
            if self.vibrato_depth > 0.0 && self.vibrato_frequency > 0.0 {
                // Why 1 - vibrato? So it's always positive?
                f += 1.0 - lerp11(0.0, self.vibrato_depth, sin_hz(self.vibrato_frequency, t));
            }

            f.max(0.0)
        }))
    }
}

/// Vibrato.
pub fn vibrato(depth: f32, frequency: f32) -> An<impl AudioNode> {
    lfo(move |t| lerp11(0.0, depth, sin_hz(frequency, t)))
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

impl Display for Tone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tone: {:?}", self.waveform)?;
        if self.interpolate_noise {
            write!(f, " interp")?;
        }
        if self.square_duty > 0.0 {
            write!(
                f,
                " duty: {:.0} sweep: {:.0}",
                self.square_duty, self.square_duty_sweep
            )?;
        }
        if self.harmonics > 0 {
            write!(
                f,
                " harmonics: {} falloff: {:.1}",
                self.harmonics, self.harmonics_falloff
            )?;
        }
        Ok(())
    }
}

impl Tone {
    pub fn pick(set: FlagSet<Waveform>, rng: &mut Rnd) -> Self {
        Self {
            waveform: Waveform::pick(set, rng),
            square_duty: if set.contains(Waveform::Square) {
                rng.f32()
            } else {
                0.0
            },
            square_duty_sweep: if set.contains(Waveform::Square) {
                rng.f32_in(-1.0, 1.0)
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

impl From<Waveform> for Tone {
    fn from(waveform: Waveform) -> Self {
        Self {
            waveform,
            ..Default::default()
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

impl Display for Amplitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "amplitude:")?;
        if self.attack > 0.0 {
            write!(f, " {:.1} attack", self.attack)?;
        }
        if self.sustain > 0.0 {
            write!(f, " {:.1} sustain", self.sustain)?;
        }
        if self.punch > 0.0 {
            write!(f, " {:.1} punch", self.punch)?;
        }
        if self.decay > 0.0 {
            write!(f, " {:.1} decay", self.decay)?;
        }
        if self.tremolo_depth > 0.0 {
            write!(
                f,
                " tremolo: {:.0}/{:.0}",
                self.tremolo_depth, self.tremolo_frequency
            )?;
        }
        Ok(())
    }
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

impl Display for Filters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.flanger_offset > 0.0 {
            write!(
                f,
                " flanger: {:.1}/{:.1}",
                self.flanger_offset, self.flanger_offset_sweep
            )?;
        }
        if self.bit_crush > 0.0 {
            write!(
                f,
                " bit_crush: {:.1}/{:.1}",
                self.bit_crush, self.bit_crush_sweep
            )?;
        }
        if self.low_pass_cutoff < 22_050.0 {
            write!(
                f,
                " low_pass: {:.0}/{:.0}",
                self.low_pass_cutoff, self.low_pass_sweep
            )?;
        }
        if self.high_pass_cutoff > 0.0 {
            write!(
                f,
                " high_pass: {:.0}/{:.0}",
                self.high_pass_cutoff, self.high_pass_sweep
            )?;
        }
        if self.compression != 1.0 {
            write!(f, " compression: {:.1}", self.compression)?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let len = 0.25;
        //let mut jump = (constant(22.0) | constant(0.5)) >> harmonic(osc::square(), 3, 0.5);

        //let mut jump = sine_hz(110.0) >> map(|i: &Frame<f32, U1>| dbg!(i[0]));
        let (mut jump, len) = powerup(0);

        //let len = 1.0;
        //let mut jump = dc(220.0 / DEFAULT_SR as f32) >> resample(white());
        //let mut jump = dc(44.1) >> osc::white(false); // >> resonator_hz(0.0, 220.0);
        // let mut jump = Pitch {
        //     frequency: 220.0,
        //     frequency_delta_sweep: -440.0,
        //     repeat_frequency: 2.0,
        //     vibrato_depth: 220.0,
        //     vibrato_frequency: 10.0,
        //     ..Default::default()
        // }
        // .to_net(1.0)
        //     >> Tone::from(Waveform::Triangle).to_net(1.0);

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
