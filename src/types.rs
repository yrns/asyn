use std::fmt;

use flagset::{flags, FlagSet};
use fundsp::hacker32::{
    clamp, clamp01, constant, dc, flanger, fract, highpole, lerp, lerp11, lfo, lfo2, lowpole,
    lowpole_hz, map, pass, pinkpass, round, sin_hz, sine, sink, An, AttoHash, AudioNode,
    AudioUnit32, Float, Frame, Net32, Num, Sine, Wave32, DEFAULT_SR, U0, U1, U2,
};
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
    pub mutations: usize,
    pub pitch: Pitch,
    pub tone: Tone,
    pub amplitude: Amplitude,
    pub filters: Option<Filters>,
}

impl Asyn {
    pub fn mutate(mut self, rng: &mut Rnd) -> Self {
        self.mutations += 1;
        self.pitch = self.pitch.mutate(rng);
        self.tone = self.tone.mutate(rng);
        self.amplitude = self.amplitude.mutate(rng);
        self.filters = Some(self.filters.unwrap_or_default().mutate(rng));
        self
    }

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
            ..
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
        write!(f, "[{}] [{}] [{}]", self.pitch, self.tone, self.amplitude)?;
        if let Some(filters) = self.filters.as_ref() {
            write!(f, "[{}]", filters)?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
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

const FREQUENCY_DEFAULT: f32 = 500.0;
const VIBRATO_FREQUENCY_DEFAULT: f32 = 10.0;
const FREQUENCY_JUMP1_ONSET_DEFAULT: f32 = 0.33;
const FREQUENCY_JUMP2_ONSET_DEFAULT: f32 = 0.66;

impl Default for Pitch {
    fn default() -> Self {
        Self {
            frequency: FREQUENCY_DEFAULT,
            frequency_sweep: 0.0,
            frequency_delta_sweep: 0.0,
            vibrato_depth: 0.0,
            vibrato_frequency: VIBRATO_FREQUENCY_DEFAULT,
            repeat_frequency: 0.0,
            frequency_jump1: (FREQUENCY_JUMP1_ONSET_DEFAULT, 0.0),
            frequency_jump2: (FREQUENCY_JUMP2_ONSET_DEFAULT, 0.0),
        }
    }
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
        if amount > 0.0 {
            write!(f, " jump1: ({onset:.2}, {amount:.2})")?;
        }
        let (onset, amount) = self.frequency_jump2;
        if amount > 0.0 {
            write!(f, " jump2: ({onset:.2}, {amount:.2})")?;
        }
        Ok(())
    }
}

/// Round to multiple of.
pub fn round_to<T: Float>(f: T, mult: T) -> T {
    round(f / mult) * mult
}

macro_rules! mutate_f32 {
    ($i:expr, $rng:expr, $def:expr, $min:literal, $max:literal, $step:literal) => {
        if $i != $def || $rng.bool(0.3) {
            let range = 0.05 * ($max - $min);
            //let prev = $i;
            $i = clamp($min, $max, round_to($i + $rng.f32_in(-range, range), $step));
            //println!("mutate_f32: {} {} -> {}", $i != $def, prev, $i);
        }
    };
}

// TODO: step for initial random values?

impl Pitch {
    pub fn mutate(mut self, rng: &mut Rnd) -> Self {
        #[rustfmt::skip]
        mutate_f32!(self.frequency, rng, FREQUENCY_DEFAULT, 10.0, 10_000.0, 100.0);
        mutate_f32!(self.frequency_sweep, rng, 0.0, -10_000.0, 10_000.0, 100.0);
        #[rustfmt::skip]
        mutate_f32!(self.frequency_delta_sweep, rng, 0.0, -10_000.0, 10_000.0, 100.0);
        mutate_f32!(self.vibrato_depth, rng, 0.0, 0.0, 1_000.0, 10.0);
        #[rustfmt::skip]
        mutate_f32!(self.vibrato_frequency, rng, VIBRATO_FREQUENCY_DEFAULT, 0.0, 1_000.0, 1.0);
        mutate_f32!(self.repeat_frequency, rng, 0.0, 0.0, 100.0, 0.1);
        #[rustfmt::skip]
        mutate_f32!(self.frequency_jump1.0, rng, FREQUENCY_JUMP1_ONSET_DEFAULT, 0.0, 1.0, 0.05);
        mutate_f32!(self.frequency_jump1.1, rng, 0.0, 0.0, 1.0, 0.05);
        #[rustfmt::skip]
        mutate_f32!(self.frequency_jump2.0, rng, FREQUENCY_JUMP2_ONSET_DEFAULT, 0.0, 1.0, 0.05);
        mutate_f32!(self.frequency_jump2.1, rng, 0.0, 0.0, 1.0, 0.05);

        self
    }

    // The first few t values are 0. Is this a bug with the envelope?
    pub fn to_net(self, len1: f32) -> Net32 {
        let erf = self.repeat_frequency.max(len1);

        wrap(lfo(move |t| {
            // t in repetition. We will get t values outside the total length because of the
            // envelope jitter, so don't actually repeat if we're not repeating.
            let t_repeat = if self.repeat_frequency > 0.0 {
                fract(t * erf)
            } else {
                t * len1
            };

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

            //println!("t: {t} t_r: {t_repeat} len: {}", (1.0 / len1));

            // Return the frequency and repeat cycle.
            (f.max(0.0), t_repeat)
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

#[derive(Clone, Debug)]
pub struct Tone {
    pub waveform: Waveform,
    pub interpolate_noise: bool,
    pub square_duty: f32,
    pub square_duty_sweep: f32,
    pub harmonics: u32,
    pub harmonics_falloff: f32,
}

impl Default for Tone {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            interpolate_noise: true,
            square_duty: 0.5,
            square_duty_sweep: 0.0,
            harmonics: 0,
            harmonics_falloff: 0.5,
        }
    }
}

impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tone: {:?}", self.waveform)?;
        if self.interpolate_noise {
            write!(f, " interp")?;
        }
        if matches!(self.waveform, Waveform::Square) && self.square_duty != 0.5 {
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
    pub fn mutate(mut self, rng: &mut Rnd) -> Self {
        // TODO: waveform from original set

        if rng.bool(0.1) {
            self.interpolate_noise = !self.interpolate_noise;
        }

        mutate_f32!(self.square_duty, rng, 0.5, 0.0, 1.0, 0.05);
        mutate_f32!(self.square_duty_sweep, rng, 0.0, -1.0, 1.0, 0.05);

        self.harmonics = clamp(0, 5, self.harmonics as i32 + i32_in(rng, -1, 1)) as u32;
        mutate_f32!(self.harmonics_falloff, rng, 0.5, 0.0, 1.0, 0.01);
        self
    }

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

    pub fn to_net(self, _len1: f32) -> Net32 {
        // The second input is the repeat cycle which only the square wave uses (for now). So every
        // other waveform gets stacked with a sink.
        let sink = wrap(sink());

        let wave = match self.waveform {
            Waveform::Sine => sine() | sink,
            Waveform::Triangle => osc::triangle() | sink,
            Waveform::Saw => osc::saw() | sink,
            Waveform::Square => {
                // Square duty sweep repeats with frequency repeat cycle.
                let duty = wrap(lfo2(move |_t, r| {
                    lerp(
                        0.01,
                        0.99,
                        self.square_duty + self.square_duty_sweep * r, //t * len1,
                    )
                }));
                (pass() | duty) >> osc::square()
            }
            Waveform::Tangent => osc::tangent() | sink,
            Waveform::Whistle => osc::whistle() | sink,
            Waveform::Breaker => osc::breaker() | sink,
            Waveform::White => osc::white(self.interpolate_noise) | sink,
            Waveform::Pink => osc::white(self.interpolate_noise) >> pinkpass() | sink,
            Waveform::Brown => {
                wrap(osc::white(self.interpolate_noise) >> lowpole_hz(10.0) * dc(13.7)) | sink
            }
        };

        if self.harmonics > 0 {
            osc::harmonic(wave, self.harmonics, self.harmonics_falloff)
        } else {
            wave
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
    pub fn mutate(mut self, rng: &mut Rnd) -> Self {
        mutate_f32!(self.attack, rng, 0.0, 0.0, 5.0, 0.01);
        mutate_f32!(self.sustain, rng, 0.0, 0.0, 5.0, 0.01);
        mutate_f32!(self.punch, rng, 0.0, 0.0, 1.0, 0.1);
        mutate_f32!(self.decay, rng, 0.0, 0.0, 5.0, 0.01);
        mutate_f32!(self.tremolo_depth, rng, 0.0, 0.0, 1.0, 0.01);
        mutate_f32!(self.tremolo_frequency, rng, 10.0, 0.0, 1000.0, 1.0);
        self
    }

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
    pub bit_crush: i32,
    pub bit_crush_sweep: i32,
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
            bit_crush: 16,
            bit_crush_sweep: 0,
            low_pass_cutoff: 22_050.0,
            low_pass_sweep: 0.0,
            high_pass_cutoff: 0.0,
            high_pass_sweep: 0.0,
            compression: 1.0,
            //normalization: true,
            //amplification: 1.0,
        }
    }
}

impl fmt::Display for Filters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.flanger_offset > 0.0 || self.flanger_offset_sweep != 0.0 {
            write!(
                f,
                " flanger: {:.1}/{:.1}",
                self.flanger_offset, self.flanger_offset_sweep
            )?;
        }
        if self.bit_crush < 16 {
            write!(f, " bit_crush: {}/{}", self.bit_crush, self.bit_crush_sweep)?;
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

// https://github.com/SamiPerttu/funutd/issues/1
#[inline]
fn i32_in(rng: &mut Rnd, min: i32, max: i32) -> i32 {
    min + rng.i32_in(0, max - min)
}

impl Filters {
    pub fn mutate(mut self, rng: &mut Rnd) -> Self {
        mutate_f32!(self.flanger_offset, rng, 0.0, 0.0, 50.0, 1.0);
        mutate_f32!(self.flanger_offset_sweep, rng, 0.0, -50.0, 50.0, 1.0);

        self.bit_crush = clamp(1, 16, self.bit_crush + i32_in(rng, -1, 1));
        self.bit_crush_sweep = clamp(-16, 16, self.bit_crush_sweep + i32_in(rng, -1, 1));

        mutate_f32!(self.low_pass_cutoff, rng, 22_050.0, 0.0, 22_050.0, 100.0);
        mutate_f32!(self.low_pass_sweep, rng, 0.0, -22_050.0, 22_050.0, 100.0);
        mutate_f32!(self.high_pass_cutoff, rng, 0.0, 0.0, 22_050.0, 100.0);
        mutate_f32!(self.high_pass_sweep, rng, 0.0, -22_050.0, 22_050.0, 100.0);

        mutate_f32!(self.compression, rng, 1.0, 0.0, 5.0, 0.1);

        // Normalization/amplification don't mutate?

        self
    }

    pub fn to_net(self, len1: f32) -> Net32 {
        let mut f = wrap(pass());

        let delay1 = self.flanger_offset;
        let sweep = self.flanger_offset_sweep;
        let delay2 = (delay1 + sweep).max(0.0);

        // jfxr does not clamp to 0 and sounds very loud without normalization. It also just sounds
        // different with a zero delay...
        if delay1 > 0.0 || delay2 > 0.0 {
            f = f
                // Make feedback a parameter?
                >> flanger(0.0, delay1.min(delay2), delay1.max(delay2), move |t| {
                    (delay1 + sweep * t * len1).max(0.0)
                });
        }

        if self.bit_crush != 0 || self.bit_crush_sweep != 0 {
            f = (f | lfo(move |t| self.bit_crush as f32 + self.bit_crush_sweep as f32 * t * len1))
                >> map(move |f: &Frame<f32, U2>| {
                    let sample = f[0];
                    let bits = clamp(1, 16, round(f[1]) as u32);
                    let steps = 2.pow(bits) as f32;
                    -1.0 + 2.0 * round((0.5 + 0.5 * sample) * steps) / steps
                });
        }

        if self.low_pass_cutoff < 22_050.0 {
            f = (f | lfo(move |t| {
                clamp(
                    0.0,
                    DEFAULT_SR as f32 / 2.0,
                    self.low_pass_cutoff + self.low_pass_sweep * t * len1,
                )
            })) >> lowpole();
        }

        if self.high_pass_cutoff > 0.0 {
            f = (f | lfo(move |t| {
                clamp(
                    0.0,
                    DEFAULT_SR as f32 / 2.0,
                    self.high_pass_cutoff + self.high_pass_sweep * t * len1,
                )
            })) >> highpole();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_duty_sweep_repeat() {
        let asyn = Asyn {
            pitch: Pitch {
                frequency: 220.0,
                repeat_frequency: 2.0,
                ..Default::default()
            },
            tone: Tone {
                waveform: Waveform::Square,
                square_duty: 0.1,
                square_duty_sweep: 0.9,
                ..Default::default()
            },
            amplitude: Amplitude {
                sustain: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };

        asyn.to_wav()
            .save_wav16("square_duty_sweep_repeat.wav")
            .unwrap();
    }

    #[test]
    fn bit_crush() {
        let asyn = Asyn {
            pitch: Pitch {
                frequency: 110.0,
                ..Default::default()
            },
            tone: Tone {
                waveform: Waveform::Triangle,
                harmonics: 4,
                harmonics_falloff: 0.9,
                ..Default::default()
            },
            amplitude: Amplitude {
                sustain: 0.1,
                ..Default::default()
            },
            filters: Some(Filters {
                bit_crush: 4,
                ..Default::default()
            }),
            ..Default::default()
        };

        asyn.to_wav().save_wav16("4bit.wav").unwrap();
    }
}
