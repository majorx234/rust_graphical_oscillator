pub mod wave_gen {
    use std::f32;

    #[derive(Debug)]
    pub struct SineWave {
        freq: u32,
        num_samples: usize,
        values: Vec<f32>,
    }

    impl SineWave {
        pub fn new(freq: u32, num_samples: usize) -> SineWave {
            let fsample_rate: f32 = 48000.0;
            let ffreq = freq as f32;

            let values_data = (0..num_samples)
                .map(|i| ((2.0 * f32::consts::PI * ffreq * (i as f32) / fsample_rate).sin()))
                .collect();
            return SineWave {
                freq: freq,
                num_samples: num_samples,
                values: values_data,
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
