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

    pub fn generate_adsr_envelope(&self, size: usize) -> Vec<f32> {
        let mut values_data: Vec<f32> = Vec::with_capacity(size);
        let fmax_attack: f32 = self.ta * size as f32;
        let fmax_decay: f32 = self.td * size as f32;
        let fmax_sustain: f32 = self.ts * size as f32;
        let fmax_release: f32 = self.tr * size as f32;

        let max_attack: u32 = fmax_attack as u32;
        let max_decay: u32 = fmax_decay as u32;
        let max_sustain: u32 = fmax_sustain as u32;
        let max_release: u32 = fmax_release as u32;

        for n in 0..max_attack {
            let s: f32 = ((n % max_attack) as f32) / fmax_attack;
            values_data.push(s);
        }
        for n in max_attack..(max_attack + max_decay) {
            let j: u32 = n - max_attack;
            let s: f32 = 1.0 - (0.7 * ((j % max_decay) as f32) / fmax_decay);
            values_data.push(s);
        }
        for _n in (max_attack + max_decay)..(max_attack + max_decay + max_sustain) {
            values_data.push(0.3);
        }
        for n in (max_attack + max_decay + max_sustain)..(size as u32) {
            let k: u32 = n - (max_attack + max_decay + max_sustain);
            let s: f32 = 0.3 - 0.3 * ((k % max_release) as f32) / fmax_release;
            values_data.push(s);
        }
        values_data
    }

    pub fn generate_adsr_note_on_envelope(&self, size: usize) -> Vec<f32> {
        let mut values_data: Vec<f32> = Vec::with_capacity(size);
        let fmax_attack: f32 = self.ta * size as f32;
        let fmax_decay: f32 = self.td * size as f32;

        let sustain_value: f32 = self.ts;

        let max_attack: u32 = fmax_attack as u32;
        let max_decay: u32 = fmax_decay as u32;

        for n in 0..max_attack {
            let s: f32 = ((n % max_attack) as f32) / fmax_attack;
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

    pub fn adsr_note_on_multiplicate(
        &self,
        in_audio: &mut [f32],
        startpose: usize,
        size: usize,
        frame_size: usize,
    ) {
        let fmax_attack: f32 = self.ta * size as f32;
        let fmax_decay: f32 = self.td * size as f32;

        let max_attack = fmax_attack as usize;
        let max_decay = fmax_decay as usize;
        let frame_factor: usize = startpose / frame_size;
        let frame_startpose = startpose % frame_size;
        let frame_max_attack: usize = max_attack % frame_size;
        let sustain_value: f32 = 0.3;
        let frame_max_decay: usize = (max_attack + max_decay) % frame_size;

        if startpose < max_attack {
            //attack
            for n in 0..frame_max_attack {
                let k: usize = startpose + n;
                let s: f32 = ((k % max_attack) as f32) / fmax_attack;
                in_audio[frame_startpose + n] *= s;
            }
            if max_decay + frame_max_attack < frame_size {
                //decay
                for n in frame_max_attack..(frame_max_attack + max_decay) {
                    let k: usize = startpose + n;
                    let s: f32 = 1.0 - (0.7 * ((k % max_decay) as f32) / fmax_decay);
                    in_audio[frame_startpose + n] *= s;
                }
                for n in (frame_max_attack + max_decay)..frame_size {
                    in_audio[frame_startpose + n] *= sustain_value;
                }
            } else {
                //decay rest of frame
                for n in frame_max_attack..frame_size {
                    let k: usize = startpose + n - max_attack;
                    let s: f32 = 1.0 - (0.7 * ((k % max_decay) as f32) / fmax_decay);
                    in_audio[frame_startpose + n] *= s;
                }
            }
        } else {
            // startpose > max_attack
            // let rest_decay = frame_size - frame_max_decay;
            //if startpose < (max_attack + max_decay - rest_decay) {
            if startpose < (max_attack + max_decay) {
                //decay
                for n in 0..frame_size {
                    let k: usize = startpose + n - max_attack;
                    let s: f32 = 1.0 - (0.7 * ((k % max_decay) as f32) / fmax_decay);
                    in_audio[frame_startpose + n] *= s;
                }
                if (max_attack + max_decay) - startpose < frame_size {
                    let rest_sustain = max_attack + max_decay - startpose;
                    for n in 0..rest_sustain {
                        in_audio[frame_startpose + n] *= sustain_value;
                    }
                }
            } else {
                //if startpose < (max_attack + max_decay + max_sustain) {
                // WIP
                // let rest_frame_size = (sample_size - startpose);
                // println!("{}", startpose);
                // println!("{}", rest_frame_size);
                // let max_frame_size = frame_size.min(frame_size - rest_frame_size);
                for n in 0..frame_size {
                    in_audio[frame_startpose + n] *= sustain_value;
                }
            }
        }
    }
    pub fn adsr_note_off_multiplicate(
        &self,
        in_audio: &mut [f32],
        startpose: usize,
        size: usize,
        frame_size: usize,
    ) {
        let fmax_release = self.tr * size as f32;
        let max_release = fmax_release as usize;
        let frame_max_release = max_release % frame_size;

        if startpose + frame_size < max_release {
            //release
            for n in 0..frame_size {
                let k: usize = startpose + n;
                let s: f32 = 0.3 - 0.3 * ((k % max_release) as f32) / fmax_release;
                in_audio[n] *= s;
            }
        } else {
            if startpose < max_release {
                for n in 0..frame_max_release {
                    let k: usize = startpose + n;
                    let s: f32 = 0.3 - 0.3 * ((k % max_release) as f32) / fmax_release;
                    in_audio[n] *= s;
                }
                // rest of frame
                for n in frame_max_release..frame_size {
                    in_audio[n] = 0.0;
                }
            } else {
                for n in 0..frame_size {
                    in_audio[n] = 0.0;
                }
            }
        }
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
            in_audio[n] *= adsr_env[n + startpose];
        }
        for n in nsamples..frame_size {
            in_audio[n] *= factor;
        }

        match note_type {
            NoteType::NoteOn => {
                let len_adsr_env = adsr_env.len() - 1;
                let max_adsr_env_index = len_adsr_env.min(startpose + nsamples - 1);
                *last_sustain_value = adsr_env[max_adsr_env_index]
            }
            NoteType::NoteOff => (),
        }
    }
}
