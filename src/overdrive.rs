use std::collections::HashMap;

use crate::effect::Effect;

pub struct Overdrive {
    pub bypassing: bool,
    symetrical: bool,
    gain: f32,
}

impl Overdrive {
    pub fn set_symetrical(&mut self) {
        self.symetrical = true;
    }
    pub fn unset_symetrical(&mut self) {
        self.symetrical = false;
    }
    pub fn set_gain(&mut self, new_gain: f32) {
        if new_gain < 0.0 {
            self.gain = 0.0;
        } else if new_gain > 10.0 {
            self.gain = 10.0;
        } else {
            self.gain = new_gain;
        }
    }
}

impl Effect for Overdrive {
    fn new() -> Self {
        Overdrive {
            bypassing: false,
            symetrical: true,
            gain: 0.0,
        }
    }

    fn name(&self) -> &'static str {
        "overdrive"
    }

    fn set_params(&mut self, params: HashMap<String, Vec<String>>) {
        // Todo
    }

    fn process_samples(
        &mut self,
        input_l: Option<&[f32]>,
        input_r: Option<&[f32]>,
        output_l: Option<&mut [f32]>,
        output_r: Option<&mut [f32]>,
    ) {
        if self.bypassing {
            if let Some(input_l) = input_l {
                if let Some(output_l) = output_l {
                    output_l.clone_from_slice(input_l);
                }
            }
            if let Some(input_r) = input_r {
                if let Some(output_r) = output_r {
                    output_r.clone_from_slice(input_r);
                }
            }
            return;
        }

        let symetrical_softclip: fn(f32, f32) -> f32 = |x: f32, gain: f32| {
            let x = x * gain;
            let sign = x.signum();
            let x = x.abs();
            if (0.0..1.0 / 3.0).contains(&x) {
                sign * 2.0 * x
            } else if 1.0 / 3.0 < x && x < 2.0 / 3.0 {
                let t = 2.0 - 3.0 * x;
                sign * (3.0 - t * t) / 3.0
            } else {
                sign * 1.0
            }
        };

        let unsymetrical_softclip: fn(f32, f32) -> f32 = |x: f32, gain: f32| {
            let x = x * gain;
            let x = x.abs();
            if (0.0..1.0 / 3.0).contains(&x) {
                2.0 * x
            } else if 1.0 / 3.0 < x && x < 2.0 / 3.0 {
                let t = 2.0 - 3.0 * x;
                (3.0 - t * t) / 3.0
            } else {
                1.0
            }
        };

        let softclip = if self.symetrical {
            symetrical_softclip
        } else {
            unsymetrical_softclip
        };

        let process_overdrive_on_slice = |input: Option<&[f32]>, output: Option<&mut [f32]>| {
            if let Some(input) = input {
                if let Some(output) = output {
                    for (index, xl) in input.iter().enumerate() {
                        output[index] = softclip(*xl, self.gain);
                    }
                    if !self.symetrical {
                        let average = output.iter().sum::<f32>() / (output.len() as f32);
                        output.iter_mut().for_each(|x| *x -= average);
                    }
                }
            }
        };

        process_overdrive_on_slice(input_l, output_l);
        process_overdrive_on_slice(input_r, output_r);
    }
    fn bypass(&mut self) {
        self.bypassing = !self.bypassing;
    }
}
