use crate::adsr::Adsr;
use crate::tone::Tone;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::tone;

#[derive(Debug)]
pub struct ToneMap {
    hm: HashMap<u32, tone::Tone>,
}

impl ToneMap {
    pub fn new() -> ToneMap {
        ToneMap { hm: HashMap::new() }
    }

    pub fn add_note_msg(&mut self, trigger_msg: TriggerNoteMsg) {
        let mut tone: Tone = Tone {
            playing: true,
            note_type: trigger_msg.note_type,
            freq: trigger_msg.freq,
            volume: trigger_msg.velocity,
            start_pose: 0,
            adsr_envelope: Adsr::new(0.1, 0.2, 0.5, 0.2),
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
                        .generate_adsr_note_on_envelope(0 as usize),
                )
            }
            NoteType::NoteOff => {
                //get last sustain value
                let (last_sustain_value_a, last_sustain_value_b) =
                    self.get_last_sustain_values_of_entry(trigger_msg.freq);

                tone.last_sustain_value_a = last_sustain_value_a;
                tone.last_sustain_value_b = last_sustain_value_b;
                self.remove(tone.freq.clone());
                //adsr_envelope.ts = last_sustain_value_a;
                tone.envelope = Some(
                    tone.adsr_envelope
                        .generate_adsr_note_off_envelope(0 as usize),
                )
            }
        }
        self.insert(tone.freq, tone);
    }

    pub fn insert(&mut self, freq_index: f32, tone: tone::Tone) {
        self.hm.insert(freq_index as u32, tone);
    }

    pub fn remove(&mut self, freq_index: f32) {
        if let Entry::Occupied(o) = self.hm.entry(freq_index as u32) {
            o.remove_entry();
        }
    }

    pub fn get_last_sustain_values_of_entry(&self, freq_index: f32) -> (f32, f32) {
        let ufreq_index = freq_index as u32;
        match self.hm.get(&ufreq_index) {
            Some(tone) => return (tone.last_sustain_value_a, tone.last_sustain_value_b),
            None => (0.0, 0.0),
        }
    }

    pub fn print(&self) {
        println!("{:?}", self.hm);
    }
}
