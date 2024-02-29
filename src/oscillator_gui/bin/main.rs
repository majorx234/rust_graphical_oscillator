use crossbeam_channel::unbounded;
use eframe::egui::ViewportBuilder;
use std::{collections::HashMap, sync::mpsc};
mod oscillator_gui;
use oscillator_gui::OscillatorGui;
use oscillator_lib::{
    jackmidi::{MidiMsgAdvanced, MidiMsgGeneric},
    midi_functions::{
        parse_json_file_to_midi_functions_with_midi_msgs_advanced,
        reverse_map_midi_functions2midi_advanced_msgs,
    },
    midi_process::midi_process_fct,
};
mod jackprocess;
use bus::Bus;
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
        midi_advanced_msgs2midi_functions =
            reverse_map_midi_functions2midi_advanced_msgs(midi_functions_with_midi_advanced_msgs);
    }
    let mut tx_close_bus = Bus::new(10);
    let rx_close_bus1 = tx_close_bus.add_rx();
    let rx_close_bus2 = tx_close_bus.add_rx();
    //let (tx_close, rx1_close) = unbounded();
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
        rx_close_bus1,
        Some(tx_midi_ctrl),
        Some(midi_advanced_msgs2midi_functions),
    );

    let jack_thread = start_jack_thread(rx_close_bus2, rx_ctrl, rx_adsr, rx_trigger, midi_sender);
    let graphical_osci_app = OscillatorGui {
        freq: 440.0,
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
        tx_close: Some(tx_close_bus),
        tx_ctrl: Some(tx_ctrl),
        tx_adsr: Some(tx_adsr),
        tx_trigger: Some(tx_trigger),
        rx_note_velocity: Some(rx_note_velocity),
        rx_midi_ctrl: Some(rx_midi_ctrl),
        init_repainter_note_velocity: true,
        init_repainter_midi_ctrl: true,
    };
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([600.0, 600.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Oscillator",
        options,
        Box::new(|_cc| Box::new(graphical_osci_app)),
    );
}
