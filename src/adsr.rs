use crate::trigger_note_msg::NoteType;

#[derive(Debug, Clone)]
pub struct Adsr {
    pub ta: f32,
    pub td: f32,
    pub ts: f32,
    pub tr: f32,
}

impl Adsr {
    pub fn new(ta: f32, td: f32, ts: f32, tr: f32) -> Self {
        return Adsr {
            ta: ta,
            td: td,
            ts: ts,
            tr: tr,
        };
    }

    pub fn generate_adsr_note_on_envelope(&self, size: usize, last_value: f32) -> Vec<f32> {
        let mut values_data: Vec<f32> = Vec::with_capacity(size);
        let fmax_attack: f32 = self.ta * size as f32;
        let fmax_decay: f32 = self.td * size as f32;

        let sustain_value: f32 = self.ts;

        let max_attack: u32 = fmax_attack as u32;
        let max_decay: u32 = fmax_decay as u32;

        for n in 0..max_attack {
            let s: f32 = last_value + (1.0 - last_value) * ((n % max_attack) as f32) / fmax_attack;
            values_data.push(s);
        }
        for n in max_attack..(max_attack + max_decay) {
            let j: u32 = n - max_attack;
            let s: f32 = 1.0 - ((1.0 - sustain_value) * ((j % max_decay) as f32) / fmax_decay);
            values_data.push(s);
        }
        for _n in (max_attack + max_decay)..size as u32 {
            values_data.push(sustain_value);
        }
        values_data
    }

    pub fn generate_adsr_note_off_envelope(&self, size: usize) -> Vec<f32> {
        let mut values_data: Vec<f32> = Vec::with_capacity(size);
        let fmax_release: f32 = self.tr * size as f32;
        let sustain_value: f32 = self.ts;

        let max_release: u32 = fmax_release as u32;

        for n in 0..(max_release) {
            let k: u32 = n;
            let s: f32 = sustain_value - sustain_value * ((k % max_release) as f32) / fmax_release;
            values_data.push(s);
        }

        for _ in max_release..size as u32 {
            values_data.push(0.0);
        }
        values_data
    }

    pub fn multiply_buf(
        &self,
        in_audio: &mut [f32],
        adsr_env: &Vec<f32>,
        startpose: usize,
        size: usize,
        frame_size: usize,
        note_type: NoteType,
        last_sustain_value: &mut f32,
        velocity: f32,
    ) {
        let (sample_length, factor): (usize, f32) = match note_type {
            NoteType::NoteOn => ((size as f32 * (self.ta + self.td)) as usize, self.ts),

            NoteType::NoteOff => ((size as f32 * self.ts) as usize, 0.0),
        };

        let mut nsamples = 0;
        if (startpose + frame_size) < sample_length {
            nsamples = frame_size;
        } else {
            if startpose < sample_length {
                nsamples = (startpose + frame_size) - sample_length;
            }
        }

        for n in 0..nsamples {
            in_audio[n] *= velocity * adsr_env[n + startpose];
        }
        for n in nsamples..frame_size {
            in_audio[n] *= velocity * factor;
        }

        let len_adsr_env = adsr_env.len() - 1;
        let tone_index = if startpose + nsamples == 0 {
            0
        } else {
            startpose + nsamples - 1
        };
        let max_adsr_env_index = len_adsr_env.min(tone_index);
        *last_sustain_value = adsr_env[max_adsr_env_index];
    }
}
