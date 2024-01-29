use crate::{
    jackmidi::MidiMsg,
    trigger_note_msg::{NoteType, TriggerNoteMsg},
};
use std::convert::TryFrom;
use std::sync::mpsc;
use wmidi;

pub fn midi_process_fct(
    midi_receiver: mpsc::Receiver<MidiMsg>,
    tx_note_velocity: crossbeam_channel::Sender<TriggerNoteMsg>,
    tx_trigger: mpsc::Sender<TriggerNoteMsg>,
    rx1_close: crossbeam_channel::Receiver<bool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while let Ok(m) = midi_receiver.recv() {
            let bytes: &[u8] = &m.data;
            if let Ok(message) = wmidi::MidiMessage::try_from(bytes) {
                match message {
                    wmidi::MidiMessage::NoteOn(_, note, val) => {
                        let velocity = u8::from(val) as f32 / 127.0;
                        let note_on_msg = TriggerNoteMsg {
                            note_type: NoteType::NoteOn,
                            freq: note.to_freq_f32(),
                            velocity,
                            length: 96000,
                        };
                        tx_note_velocity.send(note_on_msg).unwrap();
                        tx_trigger.send(note_on_msg).unwrap();

                        println!("NoteOn {} at velocity {}", note, velocity);
                    }
                    wmidi::MidiMessage::NoteOff(_, note, val) => {
                        let velocity = u8::from(val) as f32 / 127.0;
                        let note_off_msg = TriggerNoteMsg {
                            note_type: NoteType::NoteOff,
                            freq: note.to_freq_f32(),
                            velocity: velocity,
                            length: 96000,
                        };
                        tx_note_velocity.send(note_off_msg.clone()).unwrap();
                        tx_trigger.send(note_off_msg).unwrap();

                        println!("NoteOff {} at velocity {}", note, velocity);
                    }
                    message => println!("{:?}", message),
                }
            }

            let mut run: bool = true;
            if let Ok(running) = rx1_close.try_recv() {
                run = running;
            };
            if !run {
                break;
            }
        }
        println!("exit midi thread\n");
    })
}
