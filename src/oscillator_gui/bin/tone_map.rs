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

    pub fn insert(&mut self, freq_index: f32, tone: tone::Tone) {
        self.hm.insert(freq_index as u32, tone);
    }

    pub fn remove(&mut self, freq_index: f32) {
        if let Entry::Occupied(o) = self.hm.entry(freq_index as u32) {
            o.remove_entry();
        }
    }

    pub fn get(&self, freq_index: f32) -> Option<&Tone> {
        let ufreq_index = freq_index as u32;
        self.hm.get(&ufreq_index)
    }

    pub fn print(&self) {
        println!("{:?}", self.hm);
    }
}
