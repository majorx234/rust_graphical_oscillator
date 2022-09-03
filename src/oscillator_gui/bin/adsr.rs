pub struct Adsr {
    ta: f32,
    td: f32,
    ts: f32,
    tr: f32,
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
        let max_release: u32 = max_sustain as u32;

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
    pub fn adsr_note_on_multiplicate(
        &self,
        in_audio: &mut [f32],
        adsr_modificator: &Vec<f32>,
        startpose: usize,
        size: usize,
        frame_size: usize,
    ) -> () {
        let mut values_data: Vec<f32> = Vec::with_capacity(size);
        let fmax_attack: f32 = self.ta * size as f32;
        let fmax_decay: f32 = self.td * size as f32;

        let max_attack = fmax_attack as usize;
        let max_decay = fmax_decay as usize;

        if startpose < max_attack {
            for n in startpose..(max_attack.min(frame_size)) {
                let s: f32 = ((n % max_attack) as f32) / fmax_attack;
                in_audio[n - startpose] *= s;
            }
        } else {
            if startpose < max_decay {
                for n in max_attack..(max_attack + max_decay).min(frame_size) {
                    let j: usize = n - max_attack;
                    let s: f32 = 1.0 - (0.7 * ((j % max_decay) as f32) / fmax_decay);
                    in_audio[n - startpose] *= s;
                }
            }
            for n in 0..frame_size {
                in_audio[n] *= 0.3;
            }
        }
    }

    pub fn adsr_note_off_multiplicate(
        &self,
        in_audio: &mut [f32],
        adsr_modificator: &Vec<f32>,
        startpose: usize,
        size: usize,
        frame_size: usize,
    ) {
    }
}
