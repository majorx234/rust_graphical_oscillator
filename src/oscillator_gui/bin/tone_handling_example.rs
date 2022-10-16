mod tone;
use tone::Tone;
mod tone_map;
use tone_map::ToneMap;
mod tone_handling;
use tone_handling::ToneHandling;
mod adsr;
use crate::adsr::Adsr;
mod trigger_note_msg;
use crate::trigger_note_msg::NoteType;

fn main() {
    let mut tone_handling = ToneHandling::new();
}
