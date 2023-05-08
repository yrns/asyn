use fundsp::hacker32::*;
use funutd::Rnd;
use numeric_array::*;

pub fn square() -> An<Square<f32>> {
    An(Square::new(DEFAULT_SR))
}

pub fn triangle() -> An<impl AudioNode<Sample = f32, Inputs = U1, Outputs = U1>> {
    An(PhaseOsc::new(DEFAULT_SR, |phase| {
        if phase < 0.25 {
            4.0 * phase
        } else if phase < 0.75 {
            2.0 - 4.0 * phase
        } else {
            -4.0 + 4.0 * phase
        }
    }))
}

pub fn saw() -> An<impl AudioNode<Sample = f32, Inputs = U1, Outputs = U1>> {
    An(PhaseOsc::new(DEFAULT_SR, |phase| {
        if phase < 0.5 {
            2.0 * phase
        } else {
            -2.0 + 2.0 * phase
        }
    }))
}

pub fn tangent() -> An<impl AudioNode<Sample = f32, Inputs = U1, Outputs = U1>> {
    An(PhaseOsc::new(DEFAULT_SR, |phase| {
        clamp(-2.0f32, 2.0f32, 0.3f32 * tan(PI as f32 * phase))
    }))
}

pub fn whistle() -> An<impl AudioNode<Sample = f32, Inputs = U1, Outputs = U1>> {
    An(PhaseOsc::new(DEFAULT_SR, |phase| {
        0.75 * sin(TAU as f32 * phase) + 0.25 * sin(40.0 * PI as f32 * phase)
    }))
}

pub fn breaker() -> An<impl AudioNode<Sample = f32, Inputs = U1, Outputs = U1>> {
    An(PhaseOsc::new(DEFAULT_SR, |phase| {
        let mut phase = phase + sqrt(0.75);
        while phase > 1.0 {
            phase -= 1.0;
        }
        -0.1 + 2.0 * abs(1.0 - phase * phase * 2.0)
    }))
}

/// Phase oscillator.
/// - Input 0: frequency in Hz.
/// - Output 0: audio.
#[derive(Clone)]
pub struct PhaseOsc<T, F: Clone> {
    f: F,
    phase: T,
    sample_duration: T,
    hash: u64,
    initial_phase: Option<T>,
}

impl<T, F> PhaseOsc<T, F>
where
    T: Real + std::fmt::Debug,
    F: FnMut(T) -> T + Clone,
{
    pub fn with_phase(sample_rate: f64, f: F, initial_phase: Option<T>) -> Self {
        let mut osc = Self {
            f,
            phase: T::zero(),
            sample_duration: T::zero(),
            hash: 0,
            initial_phase,
        };
        osc.reset(Some(sample_rate));
        osc
    }

    pub fn new(sample_rate: f64, f: F) -> Self {
        Self::with_phase(sample_rate, f, None)
    }
}

impl<T, F> AudioNode for PhaseOsc<T, F>
where
    T: Real + std::fmt::Debug,
    F: FnMut(T) -> T + Clone,
{
    const ID: u64 = 99; // ?
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.phase = match self.initial_phase {
            Some(p) => p,
            None => T::zero(), // TODO: use hash
        };

        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.phase += input[0] * self.sample_duration;
        // From Sine::tick:
        while self.phase > T::one() {
            self.phase -= T::one();
        }

        [(self.f)(self.phase)].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..size {
            self.phase += input[0][i] * self.sample_duration;
            while self.phase > T::one() {
                self.phase -= T::one();
            }

            output[0][i] = (self.f)(self.phase);
        }
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}

/// Square oscillator.
/// - Input 0: frequency in Hz.
/// - Input 1: duty cycle.
/// - Output 0: square wave.
#[derive(Default, Clone)]
pub struct Square<T: Real> {
    phase: T,
    sample_duration: T,
    hash: u64,
    initial_phase: Option<T>,
}

impl<T> Square<T>
where
    T: Real + std::fmt::Debug,
{
    pub fn new(sample_rate: f64) -> Self {
        let mut sq = Square::default();
        sq.reset(Some(sample_rate));
        sq
    }

    pub fn with_phase(sample_rate: f64, initial_phase: Option<T>) -> Self {
        let mut sq = Self {
            initial_phase,
            ..Default::default()
        };
        sq.reset(Some(sample_rate));
        sq
    }
}

impl<T> AudioNode for Square<T>
where
    T: Real + std::fmt::Debug,
{
    const ID: u64 = 100; // ?
    type Sample = T;
    type Inputs = typenum::U2;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.phase = match self.initial_phase {
            Some(p) => p,
            None => T::zero(), // TODO: use hash
        };

        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.phase += input[0] * self.sample_duration;
        // From Sine::tick:
        while self.phase > T::one() {
            self.phase -= T::one();
        }

        [if self.phase < input[1] {
            T::one()
        } else {
            -T::one()
        }]
        .into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..size {
            self.phase += input[0][i] * self.sample_duration;
            while self.phase > T::one() {
                self.phase -= T::one();
            }
            //self.phase -= self.phase.floor();

            output[0][i] = if self.phase < input[1][i] {
                T::one()
            } else {
                -T::one()
            };
        }
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}

pub fn harmonic<A>(input: A, n: u32, falloff: f32) -> Net32
where
    A: AudioUnit32 + Clone + 'static,
{
    //let mut net = Net32::new(input.inputs(), input.outputs());

    // The first input is frequency, the rest (if any) are passed.
    let inputs = input.inputs();
    assert!(inputs >= 1);
    let xf = |i| {
        // The frequency is an increasing multiple for each successive harmonic.
        let xf = Net32::wrap(Box::new(mul(1.0 + i as f32)));
        (1..inputs).fold(xf, |acc, _| acc | pass())
    };

    // Sum of amplitudes for the input and n harmonics.
    let amplitudes = (1..=n).map(|i| falloff.pow(i as f32));
    let sum = 1.0 + amplitudes.clone().sum::<f32>();

    // Returns a scaled clone of the input.
    let scaled = |a| Net32::wrap(Box::new(input.clone())) * (a / sum);

    // Bus together the input and harmonics.
    amplitudes
        .enumerate()
        .fold(scaled(1.0), |acc, (i, a)| acc & (xf(i + 1) >> scaled(a)))
}

pub fn white(lerp: bool) -> An<impl AudioNode<Sample = f32, Inputs = U1, Outputs = U1>> {
    An(Noise::<f32>::new(DEFAULT_SR, lerp))
}

/// White noise component.
/// - Input 0: frequency.
/// - Output 0: noise.
#[derive(Default, Clone)]
pub struct Noise<T> {
    values: (T, T),
    phase: T,
    sample_duration: T,
    lerp: bool, // f?
    rnd: Rnd,
    hash: u64,
}

impl<T: Float> Noise<T> {
    pub fn new(sample_rate: f64, lerp: bool) -> Self {
        let mut noise = Self {
            lerp,
            ..Default::default()
        };
        noise.reset(Some(sample_rate));
        noise
    }
}

impl<T: Float> AudioNode for Noise<T> {
    const ID: u64 = 101;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.values = (T::zero(), T::from_f64(self.rnd.f64() * 2.0 - 1.0));
        self.rnd = Rnd::from_u64(self.hash);
        self.phase = T::zero();

        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.phase += input[0] * self.sample_duration * T::from_f64(2.0); // two samples per phase

        if self.phase > T::one() {
            self.values = (self.values.1, T::from_f64(self.rnd.f64() * 2.0 - 1.0));
            self.phase -= self.phase.floor();
        }

        let value = if self.lerp {
            lerp(self.values.0, self.values.1, self.phase)
        } else {
            self.values.1
        };
        [value].into()
    }

    #[inline]
    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}
