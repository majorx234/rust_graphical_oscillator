use crate::adsr::Adsr;
use crate::ctrl_msg::CtrlMsg;
use crate::jackaudio::SineWaveGenerator;
use crate::tone::Tone;
use crate::tone_map::ToneMap;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};
use crate::wave::Wave;

#[derive(Debug)]
pub struct ToneHandling {
    tone_map: ToneMap,
}

impl ToneHandling {
    pub fn new() -> Self {
        ToneHandling {
            tone_map: ToneMap::new(),
        }
    }
    pub fn add_note_msg(
        &mut self,
        trigger_msg: TriggerNoteMsg,
        adsr_envelope: Adsr,
        frame_size: usize,
    ) {
        let mut tone: Tone = Tone {
            playing: true,
            length: trigger_msg.length,
            note_type: trigger_msg.note_type,
            freq: trigger_msg.freq,
            velocity: trigger_msg.velocity,
            start_pose: 0,
            adsr_envelope: adsr_envelope.clone(),
            envelope: None,
            last_sustain_value_a: 0.0,
            last_sustain_value_b: 0.0,
            sine_wave_generator: SineWaveGenerator::new(frame_size, 48000.0),
        };

        //get last sustain value
        (tone.last_sustain_value_a, tone.last_sustain_value_b) =
            self.get_last_sustain_values_of_entry(trigger_msg.freq);

        // Todo: Add Check if allready playing
        if let Some(sine_wave_generator) = self.get_sine_wave_generator_of_entry(trigger_msg.freq) {
            tone.sine_wave_generator = sine_wave_generator;
        };
        match trigger_msg.note_type {
            NoteType::NoteOn => {
                tone.envelope = Some(
                    tone.adsr_envelope
                        .generate_adsr_note_on_envelope(tone.length, tone.last_sustain_value_a),
                )
            }
            NoteType::NoteOff => {
                self.tone_map.remove(tone.freq.clone());
                tone.adsr_envelope.ts = tone.last_sustain_value_a;
                tone.envelope = Some(
                    tone.adsr_envelope
                        .generate_adsr_note_off_envelope(tone.length),
                )
            }
        }
        self.tone_map.insert(tone.freq, tone);
    }

    pub fn normalize_out(
        &self,
        output_l: &mut [f32],
        output_r: &mut [f32],
        multiply_output_l: &mut [f32],
        multiply_output_r: &mut [f32],
        frame_size: usize,
    ) {
        let count_tones: f32 = if self.tone_map.len() == 0 {
            0.0
        } else {
            (self.tone_map.len() - 1) as f32
        };
        for index in 0..frame_size {
            output_l[index] -= multiply_output_l[index] * count_tones;
            output_r[index] -= multiply_output_r[index] * count_tones;
        }
    }

    pub fn process_tones(
        &mut self,
        ctrl_msg: &CtrlMsg,
        output_l: &mut [f32],
        output_r: &mut [f32],
        multiply_output_l: &mut [f32],
        multiply_output_r: &mut [f32],
        frame_size: usize,
    ) {
        output_l.fill(0.0);
        output_r.fill(0.0);
        self.tone_map
            .iterate_over_tones(Box::new(|tone: &mut Tone| {
                let mut frame_l: Vec<f32> = vec![0.0; frame_size];
                let mut frame_r: Vec<f32> = vec![0.0; frame_size];

                tone.sine_wave_generator.ctrl(ctrl_msg, tone.freq);
                tone.sine_wave_generator
                    .process_samples(&mut frame_l, &mut frame_r);
                match &tone.envelope {
                    Some(envelope) => {
                        tone.adsr_envelope.multiply_buf(
                            &mut frame_l,
                            envelope,
                            tone.start_pose,
                            tone.length,
                            frame_size,
                            tone.note_type,
                            &mut tone.last_sustain_value_a,
                            tone.velocity,
                        );
                        tone.adsr_envelope.multiply_buf(
                            &mut frame_r,
                            envelope,
                            tone.start_pose,
                            tone.length,
                            frame_size,
                            tone.note_type,
                            &mut tone.last_sustain_value_b,
                            tone.velocity,
                        );
                    }
                    None => (),
                }

                for index in 0..frame_size {
                    output_l[index] += frame_l[index];
                    output_r[index] += frame_r[index];
                    multiply_output_l[index] *= frame_l[index];
                    multiply_output_r[index] *= frame_r[index];
                }

                if tone.start_pose as f32 > tone.adsr_envelope.tr * tone.length as f32
                    && tone.note_type == NoteType::NoteOff
                {
                    tone.playing = false;
                } else {
                    tone.start_pose += frame_size;
                }
            }));
    }

    pub fn get_last_sustain_values_of_entry(&self, freq_index: f32) -> (f32, f32) {
        match self.tone_map.get(freq_index) {
            Some(tone) => (tone.last_sustain_value_a, tone.last_sustain_value_b),
            None => (0.0, 0.0),
        }
    }
    pub fn get_sine_wave_generator_of_entry(&self, freq_index: f32) -> Option<SineWaveGenerator> {
        self.tone_map
            .get(freq_index)
            .map(|tone| tone.sine_wave_generator.clone())
    }
}
