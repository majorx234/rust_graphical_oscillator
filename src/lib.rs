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
            let shift = |t: f32, freq_fm: f32, fs: f32| -> f32 {
                0.11 / freq_fm * (2.0 * f32::consts::PI * t * freq_fm / fs).cos()
            };
            let values_data = (0..num_samples)
                .map(|i| {
                    ((2.0
                        * f32::consts::PI
                        * freq_base
                        * (i as f32)
                        * shift(i as f32, freq_fm, fs)
                        / fs)
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
