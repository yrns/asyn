use flagset::{flags, FlagSet};
use fundsp::hacker32::*;
use funutd::Rnd;

pub mod osc;
pub mod play;

// #[derive(Copy, Clone, Default)]
// pub struct Pitch {
//     frequency: f32,
//     frequence_sweep: f32,
//     frequence_delta_sweep: f32,
//     vibrato_depth: f32,
//     vibrato_frequency: f32,
//     repeat_frequency: f32,
//     frequency_jump1: (f32, f32),
//     frequency_jump2: (f32, f32),
// }

flags! {
    #[derive(Default)]
    enum Waveform: u32 {
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
    waveform: Waveform,
    square_duty: f32,
    square_duty_sweep: f32,
    _harmonics: u32,
    _harmonics_falloff: f32,
}

impl Tone {
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
            Waveform::White => wrap(white()),
            Waveform::Pink => wrap(pink()),
            Waveform::Brown => wrap(brown()),
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Amplitude {
    attack: f32,
    sustain: f32,
    punch: f32,
    decay: f32,
    // tremolo_depth: f32,
    // tremolo_frequency: f32,
}

impl Amplitude {
    pub fn len(&self) -> f32 {
        self.attack + self.sustain + self.decay
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

pub fn cosine() -> An<Sine<f32>> {
    An(Sine::with_phase(DEFAULT_SR, Some(0.25f32)))
}

pub fn tremolo(
    depth: f32,
    frequency: f32,
) -> An<impl AudioNode<Sample = f32, Inputs = U0, Outputs = U1>> {
    1.0 - (depth * (0.5 + 0.5 * (constant(frequency) >> cosine())))
}

// pub fn amplitude() -> (impl AudioUnit32, f32) {
//     todo!()
// }

pub fn wrap(unit: impl AudioUnit32 + 'static) -> Net32 {
    let unit: Box<dyn AudioUnit32> = Box::new(unit);
    Net32::wrap(unit)
}

pub fn jump(seed: u32) -> (Net32, f32) {
    let mut rng = Rnd::from_u32(seed);

    let a = Amplitude {
        sustain: rng.f32_in(0.02, 0.1),
        decay: rng.f32_in(0.05, 0.4),
        punch: match rng.bool(0.5) {
            true => rng.f32(),
            false => 0.0,
        },
        ..Default::default()
    };

    let len1 = 1.0 / a.len();

    let frequency = rng.f32_in(100.0, 2000.0);
    let frequency_sweep = rng.f32_in(200.0, 2000.0);
    let pitch = wrap(lfo(move |t| frequency + frequency_sweep * t * len1));

    let tone = Tone {
        waveform: Waveform::pick(
            Waveform::Sine | Waveform::Square | Waveform::Whistle | Waveform::Breaker,
            &mut rng,
        ),
        square_duty: rng.f32_in(0.0, 100.0),
        square_duty_sweep: rng.f32_in(-100.0, 100.0),
        ..Default::default()
    };

    //let tremolo = tremolo(a.tremolo_depth, a.tremolo_frequency);
    let amplitude = wrap(lfo(move |t| aspd(a, t))); // * tremolo;

    let mut jump = pitch >> tone.to_net(len1) * amplitude;

    // Flanger. Make feedback a parameter?
    match rng.bool(0.3) {
        true => {
            let flanger_delay = rng.f32_in(0.0, 10.0);
            let flanger_sweep = rng.f32_in(-10.0, 10.0);

            jump = jump
                >> flanger(0.0, 0.0, 0.1, move |t| {
                    flanger_delay + flanger_sweep * t * len1
                });
        }
        _ => (),
    };

    // Low pass filter.
    match rng.bool(0.5) {
        true => {
            let lowpass_cutoff = 440.0;
            let lowpass_sweep = 0.0;
            let cutoff = wrap(lfo(move |t| lowpass_cutoff + lowpass_sweep * t * len1));
            jump = (jump | cutoff) >> wrap(lowpole())
        }
        _ => (),
    }

    // High pass filter.
    match rng.bool(0.5) {
        true => {
            let highpass_cutoff = 440.0;
            let highpass_sweep = 0.0;
            let cutoff = wrap(lfo(move |t| highpass_cutoff + highpass_sweep * t * len1));
            jump = (jump | cutoff) >> wrap(highpole())
        }
        _ => (),
    }

    (jump, a.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let len = 0.25;
        //let mut jump = (constant(22.0) | constant(0.5)) >> harmonic(osc::square(), 3, 0.5);

        //let mut jump = sine_hz(110.0) >> map(|i: &Frame<f32, U1>| dbg!(i[0]));
        let (mut jump, len) = jump(0);

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
