extern crate jack;
extern crate wmidi;
use bus::BusReader;
use crossbeam_channel::{Receiver, Sender};
use oscillator_lib::{
    adsr::Adsr, ctrl_msg::CtrlMsg, effect::Effect, jackmidi::MidiMsgGeneric, overdrive::Overdrive,
    tone_handling::ToneHandling, trigger_note_msg::TriggerNoteMsg,
};
use std::{process::exit, thread, time::Duration};
pub fn start_jack_thread(
    mut rx_close: BusReader<bool>,
    rx_ctrl: Receiver<CtrlMsg>,
    rx_adsr: Receiver<Adsr>,
    rx_trigger: Receiver<TriggerNoteMsg>,
    midi_sender: Sender<MidiMsgGeneric>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut run: bool = true;
        let (client, _status) =
            jack::Client::new("graphical oscillator", jack::ClientOptions::NO_START_SERVER)
                .expect("No Jack server running\n");

        let sample_rate = client.sample_rate();
        // register ports
        let mut out_a = client.register_port("gosci_out_l", jack::AudioOut).unwrap();
        let mut out_b = client.register_port("gosci_out_r", jack::AudioOut).unwrap();
        let midi_in = client.register_port("gosci_midi_in", jack::MidiIn).unwrap();

        let mut frame_size = client.buffer_size() as usize;
        if client.set_buffer_size(frame_size as u32).is_ok() {
            // get frame size
            let frame_size = client.buffer_size() as usize;
            println!(
                "client started with samplerate: {} and frame_size: {}",
                sample_rate, frame_size
            );
        } else {
            exit(-1);
        }
        if client.set_buffer_size(frame_size as u32).is_ok() {
            // get frame size
            frame_size = client.buffer_size() as usize;
            println!(
                "client started with samplerate: {} and frame_size: {}",
                sample_rate, frame_size
            );
        } else {
            exit(-1);
        }

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
            volume: 1.0,
            effect_params: None,
        };

        let mut adsr_envelope = Adsr::new(0.1, 0.2, 0.5, 0.2);
        let mut effect_in_l: Vec<f32> = vec![1.0; frame_size];
        let mut effect_in_r: Vec<f32> = vec![1.0; frame_size];

        let mut effect_chain: Vec<Box<dyn Effect>> = Vec::new();
        let mut overdrive = Overdrive::new();
        overdrive.set_gain(1.0);
        effect_chain.push(Box::new(overdrive));

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let show_p = midi_in.iter(ps);
            for e in show_p {
                let c: MidiMsgGeneric = e.into();
                let _ = midi_sender.try_send(c);
            }
            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            // zero the ringbuffer
            out_a_p.fill(0.0);
            out_b_p.fill(0.0);

            let mut multiply_out_l: Vec<f32> = vec![1.0; frame_size];
            let mut multiply_out_r: Vec<f32> = vec![1.0; frame_size];

            if let Ok(rx_ctrl_msg) = rx_ctrl.try_recv() {
                ctrl_msg = rx_ctrl_msg;
            };

            if let Ok(rx_adsr_msg) = rx_adsr.try_recv() {
                adsr_envelope = rx_adsr_msg;
            };

            if let Ok(rx_trigger_msg) = rx_trigger.try_recv() {
                tone_handling.add_note_msg(rx_trigger_msg, adsr_envelope.clone(), frame_size);
            };

            //if ctrl_msg.effect.is_some() {
            if !effect_chain.is_empty() {
                let out_a_p_inter = effect_in_l.as_mut_slice();
                let out_b_p_inter = effect_in_r.as_mut_slice();

                if let Some(ref effect_params) = ctrl_msg.effect_params {
                    effect_chain[0].set_params(effect_params);
                }

                tone_handling.process_tones(
                    &ctrl_msg,
                    out_a_p_inter,
                    out_b_p_inter,
                    &mut multiply_out_l,
                    &mut multiply_out_r,
                    frame_size,
                );

                tone_handling.normalize_out(
                    out_a_p_inter,
                    out_b_p_inter,
                    &mut multiply_out_l,
                    &mut multiply_out_r,
                    frame_size,
                );
                // if let Some(ref mut effect) = ctrl_msg.effect {
                // ToDo: check if there are new ctrl msgs for effects
                /* ToDo delete, just testing purpose
                if let Some(ref mut effect) = ctrl_msg.effect {
                    effect.process_samples(
                        Some(out_a_p_inter),
                        Some(out_b_p_inter),
                        Some(out_a_p),
                        Some(out_b_p),
                    );
                };*/

                effect_chain[0].process_samples(
                    Some(out_a_p_inter),
                    Some(out_b_p_inter),
                    Some(out_a_p),
                    Some(out_b_p),
                );
            } else {
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
            }

            jack::Control::Continue
        };

        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        while run {
            thread::sleep(Duration::from_millis(100));
            match rx_close.recv() {
                Ok(running) => run = running,
                Err(_) => run = false,
            }
        }
        match active_client.deactivate() {
            Ok(_) => println!("exit jackaudio thread\n"),
            Err(_) => println!("exit jackaudio thread,client deactivation err\n"),
        }
    })
}
