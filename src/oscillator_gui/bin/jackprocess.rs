extern crate jack;
extern crate wmidi;
use crate::adsr::Adsr;
use crate::ctrl_msg::CtrlMsg;
use crate::jackaudio::SineWaveGenerator;
use crate::jackmidi::MidiMsg;
use crate::trigger_note_msg::{NoteType, TriggerNoteMsg};
use crate::wave::Wave;
use crossbeam_channel::Receiver;
use std::{thread, time::Duration};

pub fn start_jack_thread(
    rx_close: crossbeam_channel::Receiver<bool>,
    rx_ctrl: std::sync::mpsc::Receiver<CtrlMsg>,
    rx_trigger: std::sync::mpsc::Receiver<TriggerNoteMsg>,
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

        let mut triggered: (bool, u32, NoteType) = (false, 0, NoteType::NoteOff);
        let mut set_zero: bool = false;
        let mut envelope: Option<Vec<f32>> = None;

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let adsr_envelope = Adsr::new(0.1, 0.2, 0.5, 0.2);
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
            match rx_trigger.try_recv() {
                Ok(rx_trigger_msg) => {
                    triggered = (true, rx_trigger_msg.length as u32, rx_trigger_msg.note_type);
                    envelope = Some(adsr_envelope.generate_adsr_envelope(triggered.1 as usize))
                }
                Err(_) => {}
            }
            let (playing, play_time, note_type): (bool, u32, NoteType) = triggered;

            // Use the sine_wave_generator to process samples
            if playing {
                sine_wave_generator.ctrl(&msg);
                sine_wave_generator.process_samples(out_a_p, out_b_p);
                match &envelope {
                    Some(envelope_vec) => {
                        let length = (play_time.min(frame_size)) as usize;
                        let startpose: usize = (sound_length - play_time) as usize;

                        // left channel
                        adsr_envelope.adsr_note_on_multiplicate(
                            out_a_p,
                            envelope_vec,
                            startpose,
                            length,
                            frame_size as usize,
                        );
                        adsr_envelope.adsr_note_off_multiplicate(
                            out_a_p,
                            envelope_vec,
                            startpose,
                            length,
                            frame_size as usize,
                        );

                        // left channel
                        adsr_envelope.adsr_note_on_multiplicate(
                            out_b_p,
                            envelope_vec,
                            startpose,
                            length,
                            frame_size as usize,
                        );
                        adsr_envelope.adsr_note_off_multiplicate(
                            out_b_p,
                            envelope_vec,
                            startpose,
                            length,
                            frame_size as usize,
                        );
                    }
                    None => {}
                }
            } else {
                if set_zero == true {
                    out_a_p.fill(0.0);
                    out_b_p.fill(0.0);
                    set_zero = false;
                }
            }
            if playing {
                if play_time > frame_size {
                    triggered = (true, play_time - frame_size, note_type.clone());
                } else {
                    triggered = (false, 0, note_type);
                    set_zero = true;
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
