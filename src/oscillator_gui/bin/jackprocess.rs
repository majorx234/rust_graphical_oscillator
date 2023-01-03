extern crate jack;
extern crate wmidi;
use oscillator_lib::adsr::Adsr;
use oscillator_lib::ctrl_msg::CtrlMsg;
use oscillator_lib::jackmidi::MidiMsg;
use oscillator_lib::tone_handling::ToneHandling;
use oscillator_lib::trigger_note_msg::TriggerNoteMsg;
use std::{thread, time::Duration};

pub fn start_jack_thread(
    rx_close: crossbeam_channel::Receiver<bool>,
    rx_ctrl: std::sync::mpsc::Receiver<CtrlMsg>,
    rx_adsr: std::sync::mpsc::Receiver<Adsr>,
    rx_trigger: std::sync::mpsc::Receiver<TriggerNoteMsg>,
    midi_sender: std::sync::mpsc::SyncSender<MidiMsg>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut run: bool = true;
        let (client, _status) =
            jack::Client::new("graphical oscillator", jack::ClientOptions::NO_START_SERVER)
                .expect("No Jack server running\n");

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
        let frame_size = client.buffer_size() as usize;
        let mut tone_handling = ToneHandling::new();
        let mut ctrl_msg = CtrlMsg {
            size: 0,
            intensity_am: 0.0,
            freq_am: 0.0,
            phase_am: 0.0,
            intensity_fm: 0.0,
            freq_fm: 0.0,
            phase_fm: 0.0,
            num_samples: frame_size,
        };
        let sound_length = 96000; // value of length of a synth sample
                                  //TODO:  paramter in gui or depending of midi touched key
        let mut adsr_envelope = Adsr::new(0.1, 0.2, 0.5, 0.2);

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let show_p = midi_in.iter(ps);
            for e in show_p {
                let c: MidiMsg = e.into();
                let _ = midi_sender.try_send(c);
            }
            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            let mut multiply_out_l: Vec<f32> = vec![1.0; frame_size];
            let mut multiply_out_r: Vec<f32> = vec![1.0; frame_size];

            if let Ok(rx_ctrl_msg) = rx_ctrl.try_recv() {
                ctrl_msg = rx_ctrl_msg;
            };

            if let Ok(rx_adsr_msg) = rx_adsr.try_recv() {
                adsr_envelope = rx_adsr_msg;
            };

            if let Ok(rx_trigger_msg) = rx_trigger.try_recv() {
                tone_handling.add_note_msg(rx_trigger_msg, adsr_envelope.clone());
            };

            tone_handling.process_tones(
                &ctrl_msg,
                out_a_p,
                out_b_p,
                &mut multiply_out_l,
                &mut multiply_out_r,
                frame_size,
            );

            tone_handling.normalize_out(
                out_a_p,
                out_b_p,
                &mut multiply_out_l,
                &mut multiply_out_r,
                frame_size,
            );

            jack::Control::Continue
        };

        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        while run {
            thread::sleep(Duration::from_millis(100));
            match rx_close.recv() {
                Ok(running) => (run = running),
                Err(_) => (run = false),
            }
        }
        match active_client.deactivate() {
            Ok(_) => println!("exit audio thread\n"),
            Err(_) => println!("exit audio thread,client deactivation err\n"),
        }
    })
}
