extern crate jack;
use std::sync::mpsc;
use std::{thread, time::Duration};
mod oscillator_gui;
use oscillator_gui::OscillatorGui;

fn main() {
    let (tx, rx) = mpsc::channel();
    let audio_thread = start_audio_thread(rx);
    let plot_app = OscillatorGui {
        size: 0,
        freq: 44.0,
        intensity_am: 1.0,
        freq_am: 0.0,
        intensity_fm: 1.0,
        freq_fm: 0.0,
        num_samples: 48000,
        tx: Some(tx),
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native("Oscillator", options, Box::new(|_cc| Box::new(plot_app)));
    audio_thread.join().unwrap();
}

fn start_audio_thread(rx: std::sync::mpsc::Receiver<bool>) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let (client, _status) =
            jack::Client::new("graphical oscillator", jack::ClientOptions::NO_START_SERVER)
                .unwrap();

        // register ports
        let mut out_a = client
            .register_port("gosci_out_l", jack::AudioOut::default())
            .unwrap();
        let mut out_b = client
            .register_port("gosci_out_r", jack::AudioOut::default())
            .unwrap();

        // sinewave generator
        // let sine_wave_generator

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            // Use the overdrive to process samples
            //sine_wave_generator.process_samples(out_a_p, out_b_p);

            jack::Control::Continue
        };
        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        let mut run: bool = true;
        while run {
            thread::sleep(Duration::from_millis(100));
            run = rx.recv().unwrap();
            println!("running: {}", run);
        }
        active_client.deactivate().unwrap();
    })
}
