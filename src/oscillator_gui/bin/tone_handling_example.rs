mod tone;
use tone::Tone;
mod tone_map;
use tone_map::ToneMap;
mod ctrl_msg;
use crate::ctrl_msg::CtrlMsg;
mod tone_handling;
use tone_handling::ToneHandling;
mod adsr;
use crate::adsr::Adsr;
mod trigger_note_msg;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};

fn main() {
    let mut tone_handling = ToneHandling::new();

    let trigger_msg1 = TriggerNoteMsg {
        note_type: NoteType::NoteOn,
        freq: 440.0,
        velocity: 127.0,
        length: 96000,
    };
    let adsr_envelope1: Adsr = Adsr::new(0.1, 0.2, 0.3, 0.3);

    tone_handling.add_note_msg(trigger_msg1, adsr_envelope1);

    let ctrl_msg = CtrlMsg {
        size: 96000,
        intensity_am: 1.0,
        freq_am: 0.0,
        phase_am: 0.0,
        intensity_fm: 0.0,
        freq_fm: 0.0,
        phase_fm: 0.0,
        num_samples: 96000,
    };

    for _ in [0..10] {
        let mut out_l: [f32; 1024] = [0.0; 1024];
        let mut out_r: [f32; 1024] = [0.0; 1024];
        tone_handling.process_tones(&ctrl_msg, &mut out_l, &mut out_r);
    }

    let trigger_msg2 = TriggerNoteMsg {
        note_type: NoteType::NoteOff,
        freq: 440.0,
        velocity: 0.0,
        length: 96000,
    };
    let adsr_envelope2: Adsr = Adsr::new(0.1, 0.2, 0.3, 0.3);

    tone_handling.add_note_msg(trigger_msg2, adsr_envelope2);

    for _ in [0..10] {
        let mut out_l: [f32; 1024] = [0.0; 1024];
        let mut out_r: [f32; 1024] = [0.0; 1024];
        tone_handling.process_tones(&ctrl_msg, &mut out_l, &mut out_r);
    }
}
