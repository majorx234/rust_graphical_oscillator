extern crate jack;
extern crate wmidi;
use crate::ctrl_msg::CtrlMsg;
use crate::jackaudio::SineWaveGenerator;
use crate::jackmidi::MidiMsg;
use crate::wave::Wave;
use crossbeam_channel::Receiver;
use std::{thread, time::Duration};

pub fn start_jack_thread(
    rx_close: crossbeam_channel::Receiver<bool>,
    rx_ctrl: std::sync::mpsc::Receiver<CtrlMsg>,
    rx_trigger: std::sync::mpsc::Receiver<()>,
    midi_sender: std::sync::mpsc::SyncSender<MidiMsg>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
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
        let sound_length = 96000; // value of length of a synth sample
                                  //TODO:  paramter in gui or depending of midi touched key

        let mut triggered: (bool, u32) = (false, 0);

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
            match rx_trigger.try_recv() {
                Ok(_) => triggered = (true, sound_length.clone()),
                Err(_) => {}
            }
            let (playing, play_time): (bool, u32) = triggered;

            // Use the sine_wave_generator to process samples
            if playing {
                sine_wave_generator.ctrl(&msg);
                sine_wave_generator.process_samples(out_a_p, out_b_p);
            } else {
                out_a_p.fill(0.0);
                out_b_p.fill(0.0);
            }
            if playing {
                if play_time > frame_size {
                    triggered = (true, play_time - frame_size);
                } else {
                    triggered = (false, 0);
                }
            }
            jack::Control::Continue
        };

        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        let mut run: bool = true;
        while run {
            thread::sleep(Duration::from_millis(100));
            run = rx_close.recv().unwrap();
        }
        println!("exit audio thread\n");
        active_client.deactivate().unwrap();
    })
}
