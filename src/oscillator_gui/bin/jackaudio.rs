use crate::ctrl_msg::CtrlMsg;
use crate::wave::Wave;
use oscillator_lib::wave_gen::SineWave;

pub struct SineWaveGenerator {
    pub freq: f32,
    pub intensity_am: f32,
    pub freq_am: f32,
    pub intensity_fm: f32,
    pub freq_fm: f32,
    pub offset: usize,
    pub frame_size: u32,
    pub fs: f32,
}

impl Wave for SineWaveGenerator {
    fn new(frame_size: u32, sample_rate: f32) -> Self {
        SineWaveGenerator {
            freq: 0.0,
            intensity_am: 0.0,
            freq_am: 0.0,
            intensity_fm: 0.0,
            freq_fm: 0.0,
            offset: 0,
            frame_size: frame_size,
            fs: sample_rate,
        }
    }

    fn process_samples(&mut self, output_l: &mut [f32], output_r: &mut [f32]) {
        let i: usize = 0;
        let my_sine = SineWave::new(
            self.freq,
            self.intensity_am,
            self.freq_am,
            self.intensity_fm,
            self.freq_fm,
            self.fs,
            self.frame_size as usize,
            self.offset,
        );
        let (values_size, values_data) = my_sine.gen_values();

        for i in 0..self.frame_size as usize {
            output_l[i] = values_data[i];
            output_r[i] = values_data[i];
        }
        self.offset += self.frame_size as usize;
    }

    fn ctrl(&mut self, msg: &CtrlMsg) {
        self.freq = msg.freq;
        self.intensity_am = msg.intensity_am;
        self.freq_am = msg.freq_am;
        self.intensity_fm = msg.intensity_fm;
        self.freq_fm = msg.freq_fm;
        self.offset = 0;
    }
}
