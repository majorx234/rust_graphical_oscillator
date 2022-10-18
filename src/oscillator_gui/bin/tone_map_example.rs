mod tone;
use tone::Tone;
mod tone_map;
use tone_map::ToneMap;
mod adsr;
use crate::adsr::Adsr;
mod trigger_note_msg;
use crate::trigger_note_msg::NoteType;
mod ctrl_msg;
mod jackaudio;
use crate::jackaudio::SineWaveGenerator;
mod wave;
use crate::wave::Wave;

fn main() {
    let mut tone_map = ToneMap::new();

    let new_tone = Tone {
        playing: true,
        length: 96000,
        note_type: NoteType::NoteOn,
        freq: 440.0,
        volume: 0.9,
        start_pose: 0,
        adsr_envelope: Adsr::new(0.1, 0.2, 0.5, 0.2),
        envelope: None,
        last_sustain_value_a: 0.3,
        last_sustain_value_b: 0.3,
        sine_wave_generator: SineWaveGenerator::new(1024, 48000.0),
    };

    tone_map.insert(440.0, new_tone);
    tone_map.print();

    tone_map.remove(440.0);
    tone_map.print();
}
