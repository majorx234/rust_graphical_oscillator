use std::f64;

#[derive(Debug)]
pub struct SineWave {
    freq_base: f64,
    amplitude: f64,
    intensity_am: f64,
    freq_am: f64,
    phase_am: f64,
    intensity_fm: f64,
    freq_fm: f64,
    phase_fm: f64,
    num_samples: usize,
    offset: usize,
    fs: f64,
}

impl SineWave {
    pub fn new(
        freq_base: f64,
        amplitude: f64,
        intensity_am: f64,
        freq_am: f64,
        phase_am: f64,
        intensity_fm: f64,
        freq_fm: f64,
        phase_fm: f64,
        fs: f64,
        num_samples: usize,
        offset: usize,
    ) -> SineWave {
        return SineWave {
            freq_base: freq_base,
            amplitude: amplitude,
            intensity_am: intensity_am,
            freq_am: freq_am,
            phase_am: phase_am,
            intensity_fm: intensity_fm,
            freq_fm: freq_fm,
            phase_fm: phase_fm,
            num_samples: num_samples,
            offset: offset,
            fs: fs,
        };
    }

    pub fn gen_values(&self) -> (usize, std::vec::Vec<f32>) {
        let modulator_hub: f64 = self.intensity_fm;
        let modulator_freq: f64 = self.freq_fm;
        let modulator_index: f64 = if self.freq_fm == 0.0 {
            0.0
        } else {
            modulator_hub / modulator_freq
        };

        let phase_am: f64 = self.phase_am;
        let phase_fm: f64 = self.phase_fm;
        let amp = |t: f64, freq_am: f64, fs: f64| -> f64 {
            0.5 * (self.intensity_am
                + self.intensity_am * (2.0 * f64::consts::PI * t * freq_am / fs + phase_am).cos())
        };
        let shift = |t: f64, freq_fm: f64, fs: f64| -> f64 {
            (2.0 * f64::consts::PI * t * freq_fm / fs + phase_fm).cos()
        };
        let values: Vec<f32> = (self.offset..(self.offset + self.num_samples))
            .map(|i| {
                (self.amplitude
                * ((amp(i as f64, self.freq_am, self.fs) + (1.0 - self.intensity_am))
                    * ((2.0 * f64::consts::PI * (self.freq_base / self.fs) * (i as f64)
                        + modulator_index * shift(i as f64, self.freq_fm, self.fs))
                    .sin()))) as f32
            })
            .collect();
        return (self.num_samples, values);
    }
}
