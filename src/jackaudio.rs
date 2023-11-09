use crate::ctrl_msg::CtrlMsg;
use crate::wave::Wave;
use crate::wave_gen::SineWave;

#[derive(Debug, Clone)]
pub struct SineWaveGenerator {
    pub freq: f32,
    pub amplitude: f32,
    pub intensity_am: f32,
    pub freq_am: f32,
    pub phase_am: f32,
    pub intensity_fm: f32,
    pub freq_fm: f32,
    pub phase_fm: f32,
    pub offset: usize,
    pub frame_size: usize,
    pub fs: f32,
}

impl Wave for SineWaveGenerator {
    fn new(frame_size: usize, sample_rate: f32) -> Self {
        SineWaveGenerator {
            freq: 0.0,
            amplitude: 1.0,
            intensity_am: 0.0,
            freq_am: 0.0,
            phase_am: 0.0,
            intensity_fm: 0.0,
            freq_fm: 0.0,
            phase_fm: 0.0,
            offset: 0,
            frame_size,
            fs: sample_rate,
        }
    }

    fn process_samples(&mut self, output_l: &mut [f32], output_r: &mut [f32]) {
        let my_sine = SineWave::new(
            self.freq as f64,
            self.amplitude as f64,
            self.intensity_am as f64,
            self.freq_am as f64,
            self.phase_am as f64,
            self.intensity_fm as f64,
            self.freq_fm as f64,
            self.phase_fm as f64,
            self.fs as f64,
            self.frame_size,
            self.offset,
        );
        let (_, values_data) = my_sine.gen_values();

        for i in 0..self.frame_size {
            output_l[i] = self.amplitude * values_data[i];
            output_r[i] = self.amplitude * values_data[i];
        }
        self.offset += self.frame_size;
    }

    fn ctrl(&mut self, msg: &CtrlMsg, freq: f32) {
        self.freq = freq;
        self.amplitude = msg.volume;
        self.intensity_am = msg.intensity_am;
        self.freq_am = msg.freq_am;
        self.phase_am = msg.phase_am;
        self.intensity_fm = msg.intensity_fm;
        self.freq_fm = msg.freq_fm;
        self.phase_fm = msg.phase_fm;
    }
}
