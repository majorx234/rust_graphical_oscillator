use std::f32;

#[derive(Debug)]
pub struct SineWave {
    freq_base: f32,
    intensity_am: f32,
    freq_am: f32,
    phase_am: f32,
    intensity_fm: f32,
    freq_fm: f32,
    phase_fm: f32,
    num_samples: usize,
    offset: usize,
    fs: f32,
}

impl SineWave {
    pub fn new(
        freq_base: f32,
        intensity_am: f32,
        freq_am: f32,
        phase_am: f32,
        intensity_fm: f32,
        freq_fm: f32,
        phase_fm: f32,
        fs: f32,
        num_samples: usize,
        offset: usize,
    ) -> SineWave {
        return SineWave {
            freq_base: freq_base,
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
        let modulator_hub: f32 = self.intensity_fm;
        let modulator_freq: f32 = self.freq_fm;
        let modulator_index: f32 = if self.freq_fm == 0.0 {
            0.0
        } else {
            modulator_hub / modulator_freq
        };

        let phase_am: f32 = self.phase_am;
        let phase_fm: f32 = self.phase_fm;
        let amp = |t: f32, freq_am: f32, fs: f32| -> f32 {
            0.5 * (self.intensity_am
                + self.intensity_am * (2.0 * f32::consts::PI * t * freq_am / fs + phase_am).cos())
        };
        let shift = |t: f32, freq_fm: f32, fs: f32| -> f32 {
            (2.0 * f32::consts::PI * t * freq_fm / fs + phase_fm).cos()
        };
        let values: Vec<f32> = (self.offset..(self.offset + self.num_samples))
            .map(|i| {
                (amp(i as f32, self.freq_am, self.fs) + (1.0 - self.intensity_am))
                    * ((2.0 * f32::consts::PI * (self.freq_base / self.fs) * (i as f32)
                        + modulator_index * shift(i as f32, self.freq_fm, self.fs))
                    .sin())
            })
            .collect();
        return (self.num_samples, values);
    }
}
