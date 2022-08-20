extern crate wmidi;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::convert::TryFrom;
use std::sync::mpsc;
use std::{thread, time::Duration};
mod ctrl_msg;
mod oscillator_gui;
mod wave;
use oscillator_gui::OscillatorGui;
mod jackmidi;
use jackmidi::MidiMsg;
mod jackaudio;
mod jackprocess;
use jackprocess::start_jack_thread;

fn main() {
    let (tx_close, rx1_close) = unbounded();
    let rx2_close = rx1_close.clone();
    let (tx_ctrl, rx_ctrl) = mpsc::channel();
    let (tx_note_volume, rx_note_volume): (
        std::sync::mpsc::Sender<(f32, f32)>,
        std::sync::mpsc::Receiver<(f32, f32)>,
    ) = mpsc::channel();
    let (midi_sender, midi_receiver): (
        std::sync::mpsc::SyncSender<MidiMsg>,
        std::sync::mpsc::Receiver<MidiMsg>,
    ) = mpsc::sync_channel(64);

    // midi msg test thread
    // TODO: remove later
    let midi_thread: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        while let Ok(m) = midi_receiver.recv() {
            let bytes: &[u8] = &m.data;
            let message = wmidi::MidiMessage::try_from(bytes);

            if let Ok(wmidi::MidiMessage::NoteOn(_, note, val)) = message {
                let volume = u8::from(val) as f32 / 127.0;
                let note_freq = note.to_freq_f32();
                tx_note_volume.send((note_freq, volume)).unwrap();
                println!("Singing {} at volume {}", note, volume);
            }
            println!("{:?}", m);
            let mut run = true;
            run = rx1_close.try_recv().unwrap();
            if !run {
                break;
            }
        }
        println!("exit midi thread\n");
    });

    let jack_thread = start_jack_thread(rx2_close, rx_ctrl, midi_sender);
    let graphical_osci_app = OscillatorGui {
        freq: 44.0,
        intensity_am: 1.0,
        freq_am: 0.0,
        phase_am: 0.0,
        intensity_fm: 1.0,
        freq_fm: 0.0,
        phase_fm: 0.0,
        num_samples: 48000,
        tx_close: Some(tx_close),
        tx_ctrl: Some(tx_ctrl),
        rx_note_volume: Some(rx_note_volume),
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Oscillator",
        options,
        Box::new(|_cc| Box::new(graphical_osci_app)),
    );

    midi_thread.join().unwrap();
    jack_thread.join().unwrap();
}
