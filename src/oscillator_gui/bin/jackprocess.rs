extern crate jack;
extern crate wmidi;
use crate::adsr::Adsr;
use crate::ctrl_msg::CtrlMsg;
use crate::jackaudio::SineWaveGenerator;
use crate::jackmidi::MidiMsg;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};
use crate::wave::Wave;
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
        // sinewave generator
        let mut sine_wave_generator = SineWaveGenerator::new(frame_size as u32, sample_rate as f32);
        let mut msg = CtrlMsg {
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

        let mut triggered: (bool, Option<usize>, NoteType, f32) =
            (false, None, NoteType::NoteOff, 0.0);
        let mut set_zero: bool = false;
        let mut envelope: Option<Vec<f32>> = None;
        let mut adsr_envelope = Adsr::new(0.1, 0.2, 0.5, 0.2);

        let mut last_sustain_value_a: f32 = adsr_envelope.ts;
        let mut last_sustain_value_b: f32 = adsr_envelope.ts;

        let mut startpose: usize = 0;

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let show_p = midi_in.iter(ps);
            for e in show_p {
                let c: MidiMsg = e.into();
                let _ = midi_sender.try_send(c);
            }
            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            match rx_ctrl.try_recv() {
                Ok(rx_ctrl_msg) => msg = rx_ctrl_msg,
                Err(_) => {}
            };
            match rx_adsr.try_recv() {
                Ok(rx_adsr_msg) => adsr_envelope = rx_adsr_msg,
                Err(_) => {}
            };
            match rx_trigger.try_recv() {
                Ok(rx_trigger_msg) => {
                    triggered = (true, None, rx_trigger_msg.note_type, rx_trigger_msg.freq);
                    match triggered.2 {
                        NoteType::NoteOn => {
                            envelope = Some(
                                adsr_envelope.generate_adsr_note_on_envelope(rx_trigger_msg.length),
                            );
                        }
                        NoteType::NoteOff => {
                            adsr_envelope.ts = last_sustain_value_a;
                            envelope = Some(
                                adsr_envelope
                                    .generate_adsr_note_off_envelope(rx_trigger_msg.length),
                            );
                            triggered.1 = Some(rx_trigger_msg.length);
                        }
                    };
                    startpose = 0;
                }
                Err(_) => {}
            }
            let (playing, play_time, note_type, freq): (bool, Option<usize>, NoteType, f32) =
                triggered;

            // Use the sine_wave_generator to process samples
            if playing {
                sine_wave_generator.ctrl(&msg, freq);
                sine_wave_generator.process_samples(out_a_p, out_b_p);

                match &envelope {
                    Some(envelope_vec) => {
                        adsr_envelope.multiply_buf(
                            out_a_p,
                            &envelope_vec,
                            startpose,
                            sound_length as usize,
                            frame_size,
                            note_type,
                            &mut last_sustain_value_a,
                        );
                        adsr_envelope.multiply_buf(
                            out_b_p,
                            &envelope_vec,
                            startpose,
                            sound_length as usize,
                            frame_size,
                            note_type,
                            &mut last_sustain_value_b,
                        );
                    }
                    None => {}
                }
                match play_time {
                    Some(play_time) => {
                        if play_time > frame_size {
                            triggered =
                                (true, Some(play_time - frame_size), note_type.clone(), freq);
                        } else {
                            triggered = (false, None, note_type.clone(), freq);
                            set_zero = true;
                        }
                    }
                    None => (),
                }
            } else {
                // not playing
                if set_zero == true {
                    out_a_p.fill(0.0);
                    out_b_p.fill(0.0);
                    set_zero = false;
                }
            }
            startpose += frame_size;
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
