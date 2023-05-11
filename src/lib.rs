mod osc;
mod play;
mod types;
pub mod presets {
    pub mod explosion;
    pub mod jump;
    pub mod powerup;

    pub use explosion::*;
    pub use jump::*;
    pub use powerup::*;
}

pub use osc::*;
pub use play::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use fundsp::{prelude::AudioUnit32, wave::Wave32};

    use crate::presets::*;

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
