use crossbeam_channel::unbounded;
use eframe;
use std::sync::mpsc;
mod oscillator_gui;
use oscillator_gui::OscillatorGui;
use oscillator_lib::{
    jackmidi::{MidiMsg, MidiMsgGeneric},
    midi_process::midi_process_fct,
};
mod jackprocess;
use jackprocess::start_jack_thread;
use oscillator_lib::trigger_note_msg::TriggerNoteMsg;

fn main() {
    let (tx_close, rx1_close) = unbounded();
    let rx2_close = rx1_close.clone();
    let (tx_ctrl, rx_ctrl) = mpsc::channel();
    let (tx_adsr, rx_adsr) = mpsc::channel();
    let (tx_trigger, rx_trigger) = mpsc::channel();
    let tx_trigger2 = tx_trigger.clone();
    let (tx_note_velocity, rx_note_velocity): (
        crossbeam_channel::Sender<TriggerNoteMsg>,
        crossbeam_channel::Receiver<TriggerNoteMsg>,
    ) = unbounded();
    let (midi_sender, midi_receiver): (
        std::sync::mpsc::SyncSender<MidiMsgGeneric>,
        std::sync::mpsc::Receiver<MidiMsgGeneric>,
    ) = mpsc::sync_channel(64);

    // midi msg test thread
    let midi_thread = midi_process_fct(midi_receiver, tx_note_velocity, tx_trigger2, rx1_close);

    let jack_thread = start_jack_thread(rx2_close, rx_ctrl, rx_adsr, rx_trigger, midi_sender);
    let graphical_osci_app = OscillatorGui {
        freq: 44.0,
        velocity: 1.0,
        volume: 1.0,
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
        jack_thread: Some(jack_thread),
        midi_thread: Some(midi_thread),
        tx_close: Some(tx_close),
        tx_ctrl: Some(tx_ctrl),
        tx_adsr: Some(tx_adsr),
        tx_trigger: Some(tx_trigger),
        rx_note_velocity: Some(rx_note_velocity),
        init_repainter: true,
    };
    let mut options = eframe::NativeOptions::default();
    let window_size: eframe::egui::Vec2 = eframe::egui::Vec2::new(800.0, 600.0);
    options.initial_window_size = Some(window_size);
    eframe::run_native(
        "Oscillator",
        options,
        Box::new(|_cc| Box::new(graphical_osci_app)),
    );
}
