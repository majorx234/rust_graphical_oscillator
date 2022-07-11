pub mod wave_gen {
    use std::f32;

    #[derive(Debug)]
    pub struct SineWave {
        freq_base: f32,
        freq_am: f32,
        freq_fm: f32,
        num_samples: usize,
        values: Vec<f32>,
        fs: f32,
    }

    impl SineWave {
        pub fn new(
            freq_base: f32,
            freq_am: f32,
            freq_fm: f32,
            fs: f32,
            num_samples: usize,
        ) -> SineWave {
            let modulator_hub: f32 = 0.001;
            let modulator_freq: f32 = freq_fm / fs;
            let modulator_index: f32 = modulator_hub / modulator_freq;
            let amp = |t: f32, freq_am: f32, fs: f32| -> f32 {
                0.5 + 0.5 * (2.0 * f32::consts::PI * t * freq_am / fs).cos()
            };
            let shift = |t: f32, freq_fm: f32, fs: f32| -> f32 {
                (2.0 * f32::consts::PI * t * freq_fm / fs).cos()
            };
            let values_data = (0..num_samples)
                .map(|i| {
                    amp(i as f32, freq_am, fs)
                        * ((2.0 * f32::consts::PI * (freq_base / fs) * (i as f32)
                            + modulator_index * shift(i as f32, freq_fm, fs))
                        .sin())
                })
                .collect();
            return SineWave {
                freq_base: freq_base,
                freq_am: freq_am,
                freq_fm: freq_fm,
                num_samples: num_samples,
                values: values_data,
                fs: fs,
            };
        }
        pub fn print(&self) -> () {
            for sample in &self.values {
                println!("{}", sample);
            }
        }
        pub fn get_values(&self) -> (usize, &std::vec::Vec<f32>) {
            return (self.num_samples, &self.values);
        }
    }
}
