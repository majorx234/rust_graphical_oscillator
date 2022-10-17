use crate::adsr::Adsr;
use crate::ctrl_msg::CtrlMsg;
use crate::tone::Tone;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};

use crate::tone_map::ToneMap;

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
    pub fn add_note_msg(&mut self, trigger_msg: TriggerNoteMsg, adsr_envelope: Adsr) {
        let mut tone: Tone = Tone {
            playing: true,
            length: trigger_msg.length,
            note_type: trigger_msg.note_type,
            freq: trigger_msg.freq,
            volume: trigger_msg.velocity,
            start_pose: 0,
            adsr_envelope: adsr_envelope,
            envelope: None,
            last_sustain_value_a: 0.3,
            last_sustain_value_b: 0.3,
        };

        match trigger_msg.note_type {
            NoteType::NoteOn => {
                // Todo: Add Check if allready playing
                tone.last_sustain_value_a = tone.adsr_envelope.ts;
                tone.last_sustain_value_b = tone.adsr_envelope.ts;

                tone.envelope = Some(
                    tone.adsr_envelope
                        .generate_adsr_note_on_envelope(tone.length),
                )
            }
            NoteType::NoteOff => {
                //get last sustain value
                let (last_sustain_value_a, last_sustain_value_b) =
                    self.get_last_sustain_values_of_entry(trigger_msg.freq);

                tone.last_sustain_value_a = last_sustain_value_a;
                tone.last_sustain_value_b = last_sustain_value_b;
                self.tone_map.remove(tone.freq.clone());
                //adsr_envelope.ts = last_sustain_value_a;
                tone.envelope = Some(
                    tone.adsr_envelope
                        .generate_adsr_note_off_envelope(tone.length),
                )
            }
        }
        self.tone_map.insert(tone.freq, tone);
    }

    pub fn process_tones(&mut self, msg: &CtrlMsg, output_l: &mut [f32], output_r: &mut [f32]) {
        self.tone_map
            .iterate_over_tones(Box::new(|tone: &Tone| println!("tone {:?}", tone)));
    }

    pub fn get_last_sustain_values_of_entry(&self, freq_index: f32) -> (f32, f32) {
        match self.tone_map.get(freq_index) {
            Some(tone) => return (tone.last_sustain_value_a, tone.last_sustain_value_b),
            None => (0.0, 0.0),
        }
    }
}
