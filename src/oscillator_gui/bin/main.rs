use crossbeam_channel::unbounded;
use eframe;
use std::convert::TryFrom;
use std::sync::mpsc;
use wmidi;
mod ctrl_msg;
mod oscillator_gui;
mod trigger_note_msg;
mod wave;
use oscillator_gui::OscillatorGui;
mod jackmidi;
use jackmidi::MidiMsg;
mod jackaudio;
mod jackprocess;
use jackprocess::start_jack_thread;
mod adsr;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};

fn main() {
    let (tx_close, rx1_close) = unbounded();
    let rx2_close = rx1_close.clone();
    let (tx_ctrl, rx_ctrl) = mpsc::channel();
    let (tx_adsr, rx_adsr) = mpsc::channel();
    let (tx_trigger, rx_trigger) = mpsc::channel();
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
            if let Ok(message) = wmidi::MidiMessage::try_from(bytes) {
                match message {
                    wmidi::MidiMessage::NoteOn(_, note, val) => {
                        let volume = u8::from(val) as f32 / 127.0;
                        let note_freq = note.to_freq_f32();
                        tx_note_volume.send((note_freq, volume)).unwrap();
                        println!("NoteOn {} at volume {}", note, volume);
                    }
                    wmidi::MidiMessage::NoteOff(_, note, val) => {
                        let volume = u8::from(val) as f32 / 127.0;
                        let note_freq = note.to_freq_f32();
                        tx_note_volume.send((note_freq, volume)).unwrap();
                        println!("NoteOff {} at volume {}", note, volume);
                    }
                    message => println!("{:?}", m),
                }
            }

            let mut run: bool = true;
            match rx1_close.try_recv() {
                Ok(running) => (run = running),
                Err(_) => (),
            }
            if !run {
                break;
            }
        }
        println!("exit midi thread\n");
    });

    let jack_thread = start_jack_thread(rx2_close, rx_ctrl, rx_adsr, rx_trigger, midi_sender);
    let graphical_osci_app = OscillatorGui {
        freq: 44.0,
        intensity_am: 1.0,
        freq_am: 0.0,
        phase_am: 0.0,
        intensity_fm: 1.0,
        freq_fm: 0.0,
        phase_fm: 0.0,
        attack: 0.1,
        decay: 0.2,
        sustain: 0.3,
        release: 0.2,
        num_samples: 48000,
        length: 96000,
        tx_close: Some(tx_close),
        tx_ctrl: Some(tx_ctrl),
        tx_adsr: Some(tx_adsr),
        tx_trigger: Some(tx_trigger),
        rx_note_volume: Some(rx_note_volume),
    };
    let mut options = eframe::NativeOptions::default();
    let window_size: eframe::egui::Vec2 = eframe::egui::Vec2::new(800.0, 600.0);
    options.initial_window_size = Some(window_size);
    eframe::run_native(
        "Oscillator",
        options,
        Box::new(|_cc| Box::new(graphical_osci_app)),
    );

    midi_thread.join().unwrap();
    jack_thread.join().unwrap();
}
