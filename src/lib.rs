mod osc;
mod play;
mod types;
pub mod presets {
    pub mod blip;
    pub mod explosion;
    pub mod hit;
    pub mod jump;
    pub mod laser;
    pub mod pickup;
    pub mod powerup;
    pub mod random;

    pub use blip::*;
    pub use explosion::*;
    pub use hit::*;
    pub use jump::*;
    pub use laser::*;
    pub use pickup::*;
    pub use powerup::*;
    pub use random::*;
}

pub use osc::*;
pub use play::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::presets::*;

    #[test]
    fn it_works() {
        //let len = 0.25;
        //let mut jump = (constant(22.0) | constant(0.5)) >> harmonic(osc::square(), 3, 0.5);

        //let mut jump = sine_hz(110.0) >> map(|i: &Frame<f32, U1>| dbg!(i[0]));

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

        let mut rng = funutd::Rnd::from_time();
        let seed = rng.u64();
        let rng = &mut funutd::Rnd::from_u64(seed);
        println!("seed: {}", seed);

        random(rng).to_wav().save_wav16("test.wav").unwrap();
        //wav.write_wav16(&mut std::io::stdout().lock()).unwrap();
    }
}
