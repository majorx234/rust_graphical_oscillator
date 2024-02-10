use crossbeam_channel::unbounded;
use eframe;
use std::{collections::HashMap, sync::mpsc};
mod oscillator_gui;
use oscillator_gui::OscillatorGui;
use oscillator_lib::{
    jackmidi::{MidiMsgAdvanced, MidiMsgGeneric},
    midi_functions::parse_json_file_to_midi_functions_with_midi_msgs_advanced,
    midi_process::midi_process_fct,
};
mod jackprocess;
use clap::Parser;
use jackprocess::start_jack_thread;
use oscillator_lib::trigger_note_msg::TriggerNoteMsg;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// midi_mapping_filepath
    #[arg(short, long, value_name = "filepath")]
    pub midi_mapping_filepath: Option<String>,
}

fn main() {
    let midi_functions_with_midi_advanced_msgs: Result<
        HashMap<String, Vec<MidiMsgAdvanced>>,
        String,
    > = Args::parse().midi_mapping_filepath.map_or_else(
        || Ok(HashMap::<String, Vec<MidiMsgAdvanced>>::new()),
        |filepath| {
            // Todo: parse filepath
            println!("found midi config file: {}", filepath.as_str());
            parse_json_file_to_midi_functions_with_midi_msgs_advanced(&filepath)
        },
    );
    // create a reverse Hashmap
    let mut midi_advanced_msgs2midi_functions: HashMap<MidiMsgAdvanced, Vec<String>> =
        HashMap::new();
    if let Ok(midi_functions_with_midi_advanced_msgs) = midi_functions_with_midi_advanced_msgs {
        for (key, value_vec) in midi_functions_with_midi_advanced_msgs {
            let key_insert = key.clone();
            for value in value_vec {
                if let Some(ref mut midi_function_vec) =
                    midi_advanced_msgs2midi_functions.get_mut(&value)
                {
                    midi_function_vec.push(key_insert.clone());
                } else {
                    midi_advanced_msgs2midi_functions.insert(value, vec![key_insert.clone()]);
                }
            }
        }
    }
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
    let (tx_midi_ctrl, rx_midi_ctrl): (
        crossbeam_channel::Sender<(String, f32)>,
        crossbeam_channel::Receiver<(String, f32)>,
    ) = unbounded();
    // midi msg test thread
    let midi_thread = midi_process_fct(
        midi_receiver,
        tx_note_velocity,
        tx_trigger2,
        rx1_close,
        Some(tx_midi_ctrl),
        Some(midi_advanced_msgs2midi_functions),
    );

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
        rx_midi_ctrl: Some(rx_midi_ctrl),
        init_repainter_note_velocity: true,
        init_repainter_midi_ctrl: true,
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
