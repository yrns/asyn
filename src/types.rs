use std::fmt;

use flagset::{flags, FlagSet};
use fundsp::hacker32::*;
use funutd::Rnd;

use crate::osc;

/// Vibrato.
pub fn vibrato(depth: f32, frequency: f32) -> An<impl AudioNode> {
    lfo(move |t| lerp11(0.0, depth, sin_hz(frequency, t)))
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

#[derive(Debug, Default)]
pub struct Asyn {
    pub seed: u64,
    pub pitch: Pitch,
    pub tone: Tone,
    pub amplitude: Amplitude,
    pub filters: Option<Filters>,
}

impl Asyn {
    pub fn len(&self) -> f32 {
        self.amplitude.len()
    }

    pub fn to_net(self) -> Net32 {
        let Asyn {
            seed,
            pitch,
            tone,
            amplitude,
            filters,
        } = self;

        let len = amplitude.len();
        let len1 = 1.0 / len;

        let mut net = pitch.to_net(len1) >> (tone.to_net(len1) * amplitude.to_net());
        if let Some(f) = filters {
            net = net >> f.to_net(len1);
        }

        // This makes it so there's no random variance with the same seed.
        net.ping(false, AttoHash::new(seed));

        net
    }

    pub fn to_wav(self) -> Wave32 {
        println!("to_wav: {}", &self);
        Wave32::render(DEFAULT_SR, self.len() as f64, &mut self.to_net())
    }
}

impl fmt::Display for Asyn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "seed: {} [{}] [{}] [{}]",
            self.seed, self.pitch, self.tone, self.amplitude
        )?;
        if let Some(filters) = self.filters.as_ref() {
            write!(f, "[{}]", filters)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
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

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0}hz", self.frequency)?;
        if self.frequency_sweep != 0.0 {
            write!(f, " sweep: {:.0}", self.frequency_sweep)?;
        }
        if self.frequency_delta_sweep != 0.0 {
            write!(f, " delta sweep: {:.0}", self.frequency_delta_sweep)?;
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
        let (onset, amount) = self.frequency_jump1;
        if onset > 0.0 {
            write!(f, " jump1: ({onset:.2}, {amount:.2})")?;
        }
        let (onset, amount) = self.frequency_jump2;
        if onset > 0.0 {
            write!(f, " jump2: ({onset:.2}, {amount:.2})")?;
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

#[derive(Clone, Debug, Default)]
pub struct Tone {
    pub waveform: Waveform,
    pub interpolate_noise: bool,
    pub square_duty: f32,
    pub square_duty_sweep: f32,
    pub harmonics: u32,
    pub harmonics_falloff: f32,
}

impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tone: {:?}", self.waveform)?;
        if self.interpolate_noise {
            write!(f, " interp")?;
        }
        if matches!(self.waveform, Waveform::Square) && self.square_duty > 0.0 {
            write!(
                f,
                " duty: {:.2} sweep: {:.2}",
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
            // Interpolation defaults to true for noise.
            interpolate_noise: set.contains(Waveform::White | Waveform::Pink | Waveform::Brown),
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

#[derive(Copy, Clone, Debug, Default)]
pub struct Amplitude {
    pub attack: f32,
    pub sustain: f32,
    pub punch: f32,
    pub decay: f32,
    pub tremolo_depth: f32,
    pub tremolo_frequency: f32,
}

impl fmt::Display for Amplitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "amplitude:")?;
        if self.attack > 0.0 {
            write!(f, " {:.2} attack", self.attack)?;
        }
        if self.sustain > 0.0 {
            write!(f, " {:.2} sustain", self.sustain)?;
        }
        if self.punch > 0.0 {
            write!(f, " {:.2} punch", self.punch)?;
        }
        if self.decay > 0.0 {
            write!(f, " {:.2} decay", self.decay)?;
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

#[derive(Clone, Debug)]
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

impl fmt::Display for Filters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        if self.low_pass_cutoff < 22_050.0 || self.low_pass_sweep != 0.0 {
            write!(
                f,
                " low_pass: {:.0}/{:.0}",
                self.low_pass_cutoff, self.low_pass_sweep
            )?;
        }
        if self.high_pass_cutoff > 0.0 || self.high_pass_sweep != 0.0 {
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

        let c = self.compression;
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
