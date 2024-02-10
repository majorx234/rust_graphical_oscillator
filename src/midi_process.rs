use crate::{
    jackmidi::{MidiMsgAdvanced, MidiMsgBase, MidiMsgGeneric},
    trigger_note_msg::{NoteType, TriggerNoteMsg},
    util::*,
};
use std::collections::HashMap;
use std::sync::mpsc;

pub fn midi_process_fct(
    midi_receiver: mpsc::Receiver<MidiMsgGeneric>,
    tx_note_velocity: crossbeam_channel::Sender<TriggerNoteMsg>,
    tx_trigger: mpsc::Sender<TriggerNoteMsg>,
    rx1_close: crossbeam_channel::Receiver<bool>,
    tx_midi_ctrl: Option<crossbeam_channel::Sender<(String, f32)>>,
    midi_advanced_msgs2midi_functions: Option<HashMap<MidiMsgAdvanced, Vec<String>>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last_midi_msg: Option<Box<dyn MidiMsgBase>> = None;
        let mut run_loop = true;

        while run_loop {
            while let Ok(msg_generic) = midi_receiver.recv() {
                let midi_msg: Box<dyn MidiMsgBase> = msg_generic.into();

                let midi_advanced_msg = MidiMsgAdvanced::from_current_and_last_opt_midi_msgs((
                    midi_msg,
                    &mut last_midi_msg,
                ));
                if let Some(midi_advanced_msg) = midi_advanced_msg {
                    let _id = midi_advanced_msg.get_id();
                    match midi_advanced_msg {
                        MidiMsgAdvanced::MidiNoteOnOff(_id0, _id1, bvalue, note, intensity) => {
                            if bvalue == true {
                                let velocity = intensity as f32 / 127.0;
                                let note_on_msg = TriggerNoteMsg {
                                    note_type: NoteType::NoteOn,
                                    freq: to_freq_f32(note),
                                    velocity,
                                    length: 96000,
                                };
                                tx_note_velocity.send(note_on_msg).unwrap();
                                tx_trigger.send(note_on_msg).unwrap();
                            } else {
                                let velocity = intensity as f32 / 127.0;
                                let note_off_msg = TriggerNoteMsg {
                                    note_type: NoteType::NoteOff,
                                    freq: to_freq_f32(note),
                                    velocity,
                                    length: 96000,
                                };
                                tx_note_velocity.send(note_off_msg.clone()).unwrap();
                                tx_trigger.send(note_off_msg).unwrap();
                            }
                        }
                        mut other_midi_advanced_msg => {
                            if let Some(ref tx_midi_ctrl) = tx_midi_ctrl {
                                if let Some(ref midi_advanced_msgs2midi_functions) =
                                    midi_advanced_msgs2midi_functions
                                {
                                    let value = other_midi_advanced_msg.get_norm_value();
                                    other_midi_advanced_msg.reset_value();
                                    if let Some(functions) = midi_advanced_msgs2midi_functions
                                        .get(&other_midi_advanced_msg)
                                    {
                                        for function in functions {
                                            let _ = tx_midi_ctrl
                                                .try_send((function.to_string(), value));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let mut run: bool = true;
            if let Ok(running) = rx1_close.try_recv() {
                run = running;
                run_loop = run;
            };
            if !run {
                break;
            }
        }
        println!("exit midi thread\n");
    })
}
