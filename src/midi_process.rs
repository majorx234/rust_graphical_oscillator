use crate::{
    jackmidi::{MidiMsgAdvanced, MidiMsgBase, MidiMsgGeneric},
    trigger_note_msg::{NoteType, TriggerNoteMsg},
    util::*,
};
use std::convert::TryFrom;
use std::sync::mpsc;
use wmidi;

pub fn midi_process_fct(
    midi_receiver: mpsc::Receiver<MidiMsgGeneric>,
    tx_note_velocity: crossbeam_channel::Sender<TriggerNoteMsg>,
    tx_trigger: mpsc::Sender<TriggerNoteMsg>,
    rx1_close: crossbeam_channel::Receiver<bool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last_midi_msg: Option<Box<dyn MidiMsgBase>> = None;
        let mut run_loop = true;

        while run_loop {
            let mut received_midi_advanced_messages: Vec<MidiMsgAdvanced> = Vec::new();

            while let Ok(msg_generic) = midi_receiver.recv() {
                let midi_msg: Box<dyn MidiMsgBase> = msg_generic.into();

                let midi_advanced_msg = MidiMsgAdvanced::from_current_and_last_opt_midi_msgs((
                    midi_msg,
                    &mut last_midi_msg,
                ));
                if let Some(midi_advanced_msg) = midi_advanced_msg {
                    let _id = midi_advanced_msg.get_id();
                    received_midi_advanced_messages.push(midi_advanced_msg);
                }
            }
            for midi_advanced_msg in received_midi_advanced_messages {
                match midi_advanced_msg {
                    MidiMsgAdvanced::MidiNoteOnOff(id0, id1, bvalue, note, intensity) => {
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
                    _ => (),
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
