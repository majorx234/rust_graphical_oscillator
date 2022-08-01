extern crate jack;
extern crate wmidi;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::sync::mpsc;
use std::sync::mpsc::sync_channel;
use std::{thread, time::Duration};
mod ctrl_msg;
mod wave;
use crate::wave::Wave;
use ctrl_msg::CtrlMsg;
mod jackaudio;
use jackaudio::SineWaveGenerator;
mod jackmidi;
use jackmidi::MidiMsg;
use wmidi::MidiMessage;
mod oscillator_gui;
use oscillator_gui::OscillatorGui;

fn main() {
    let (tx_close, rx_close) = mpsc::channel();
    let (tx_ctrl, rx_ctrl) = mpsc::channel();
    let (midi_sender, midi_receiver): (
        std::sync::mpsc::SyncSender<MidiMsg>,
        std::sync::mpsc::Receiver<MidiMsg>,
    ) = mpsc::sync_channel(64);

    // midi msg test thread
    // TODO: remove later
    std::thread::spawn(move || {
        while let Ok(m) = midi_receiver.recv() {
            let bytes: &[u8] = &m.data;
            let message = wmidi::MidiMessage::try_from(bytes);

            if let Ok(wmidi::MidiMessage::NoteOn(_, note, val)) = message {
                let volume = u8::from(val) as f32 / 127.0;
                println!("Singing {} at volume {}", note, volume);
            }
            println!("{:?}", m);
        }
    });

    let audio_thread = start_audio_thread(rx_close, rx_ctrl, midi_sender);
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
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Oscillator",
        options,
        Box::new(|_cc| Box::new(graphical_osci_app)),
    );
    audio_thread.join().unwrap();
}

fn start_audio_thread(
    rx_close: std::sync::mpsc::Receiver<bool>,
    rx_ctrl: std::sync::mpsc::Receiver<CtrlMsg>,
    midi_sender: std::sync::mpsc::SyncSender<MidiMsg>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let (client, _status) =
            jack::Client::new("graphical oscillator", jack::ClientOptions::NO_START_SERVER)
                .unwrap();
        let sample_rate = client.sample_rate();
        // register ports
        let mut out_a = client
            .register_port("gosci_out_l", jack::AudioOut::default())
            .unwrap();
        let mut out_b = client
            .register_port("gosci_out_r", jack::AudioOut::default())
            .unwrap();
        let midi_in = client
            .register_port("gosci_midi_in", jack::MidiIn::default())
            .unwrap();

        // get frame size
        let frame_size = client.buffer_size();
        // sinewave generator
        let mut sine_wave_generator = SineWaveGenerator::new(frame_size as u32, sample_rate as f32);
        let mut msg = CtrlMsg {
            size: 0,
            freq: 0.0,
            intensity_am: 0.0,
            freq_am: 0.0,
            phase_am: 0.0,
            intensity_fm: 0.0,
            freq_fm: 0.0,
            phase_fm: 0.0,
            num_samples: frame_size as usize,
        };

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let show_p = midi_in.iter(ps);
            for e in show_p {
                let c: MidiMsg = e.into();
                let _ = midi_sender.try_send(c);
            }
            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            match rx_ctrl.try_recv() {
                Ok(rx) => msg = rx,
                Err(_) => {}
            };
            // Use the sine_wave_generator to process samples
            sine_wave_generator.ctrl(&msg);
            sine_wave_generator.process_samples(out_a_p, out_b_p);
            jack::Control::Continue
        };

        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        let mut run: bool = true;
        while run {
            thread::sleep(Duration::from_millis(100));
            run = rx_close.recv().unwrap();
            println!("running: {}", run);
        }
        active_client.deactivate().unwrap();
    })
}
