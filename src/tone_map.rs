use crate::tone::Tone;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::tone;

#[derive(Debug)]
pub struct ToneMap {
    hm: HashMap<u32, tone::Tone>,
}

impl Default for ToneMap {
    fn default() -> Self {
        ToneMap::new()
    }
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

    pub fn len(&self) -> usize {
        self.hm.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hm.is_empty()
    }

    pub fn print(&self) {
        println!("{:?}", self.hm);
    }

    pub fn iterate_over_tones(&mut self, mut fnct: Box<dyn FnMut(&mut Tone) + '_>) {
        let mut tone_to_delete_key_list: Vec<u32> = Vec::new();
        for (key, value) in self.hm.iter_mut() {
            if !value.playing {
                tone_to_delete_key_list.push(*key);
            } else {
                fnct(value);
            }
        }
        for key in tone_to_delete_key_list {
            if let Entry::Occupied(o) = self.hm.entry(key) {
                o.remove_entry();
            }
        }
    }
}
