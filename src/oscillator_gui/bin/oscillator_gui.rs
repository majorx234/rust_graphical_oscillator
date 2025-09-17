use crate::status_button::status_button;
use bus::Bus;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::{
    egui::{self, PointerButton, ViewportCommand},
    glow::Context,
};
use egui_plot::{Line, Plot, PlotPoints};
use oscillator_lib::adsr::Adsr;
use oscillator_lib::ctrl_msg::{CtrlMsg, ParameterMap};
use oscillator_lib::trigger_note_msg::{NoteType, TriggerNoteMsg};
use oscillator_lib::wave_gen::SineWave;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::thread;

pub struct OscillatorGui {
    pub freq: f32,
    pub velocity: f32,
    pub volume: f32,
    pub intensity_am: f32,
    pub freq_am: f32,
    pub phase_am: f32,
    pub intensity_fm: f32,
    pub freq_fm: f32,
    pub phase_fm: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub num_samples: usize,
    pub length: usize,
    pub jack_thread: Option<std::thread::JoinHandle<()>>,
    pub midi_thread: Option<std::thread::JoinHandle<()>>,
    pub tx_close: Option<Bus<bool>>,
    pub tx_ctrl: Option<Sender<CtrlMsg>>,
    pub tx_adsr: Option<Sender<Adsr>>,
    pub tx_trigger: Option<Sender<TriggerNoteMsg>>,
    pub rx_note_velocity: Option<Receiver<TriggerNoteMsg>>,
    pub rx_midi_ctrl: Option<Receiver<(String, f32)>>,
    pub init_repainter_note_velocity: bool,
    pub init_repainter_midi_ctrl: bool,
    pub overdrive_toggle: bool,
    pub overdrive: f32,
}

impl Default for OscillatorGui {
    fn default() -> Self {
        Self {
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
            jack_thread: None,
            midi_thread: None,
            tx_close: None,
            tx_ctrl: None,
            tx_adsr: None,
            tx_trigger: None,
            rx_note_velocity: None,
            rx_midi_ctrl: None,
            init_repainter_note_velocity: true,
            init_repainter_midi_ctrl: true,
            overdrive_toggle: false,
            overdrive: 1.0,
        }
    }
}

impl eframe::App for OscillatorGui {
    /// Called once before the first frame.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.init_repainter_note_velocity {
            // singleton pattern as setup()
            let ctx = ctx.clone();
            if let Some(rx_note_velocity) = &self.rx_note_velocity {
                let rx_note_velocity2 = rx_note_velocity.clone();
                //need a chain of crossbeam channels
                let (tx_note_velocity, rx_note_velocity) = unbounded();
                thread::spawn(|| {
                    repainter::<TriggerNoteMsg>(
                        ctx,
                        Some(rx_note_velocity2),
                        Some(tx_note_velocity),
                    )
                });
                self.rx_note_velocity = Some(rx_note_velocity);
            }
            self.init_repainter_note_velocity = false;
        }
        if self.init_repainter_midi_ctrl {
            let ctx = ctx.clone();
            if let Some(ref rx_midi_ctrl) = self.rx_midi_ctrl {
                let rx_midi_ctrl2 = rx_midi_ctrl.clone();
                let (tx_midi_ctrl, rx_midi_ctrl) = unbounded();
                thread::spawn(|| {
                    repainter::<(String, f32)>(ctx, Some(rx_midi_ctrl2), Some(tx_midi_ctrl))
                });
                self.rx_midi_ctrl = Some(rx_midi_ctrl);
            }
            self.init_repainter_midi_ctrl = false;
        }
        if let Some(ref rx_midi_ctrl) = self.rx_midi_ctrl {
            let mut received_midi_ctrl_messages: Vec<(String, f32)> = Vec::new();
            while let Ok(midi_ctrl_msgs) = rx_midi_ctrl.try_recv() {
                received_midi_ctrl_messages.push(midi_ctrl_msgs);
            }
            for (function, value) in received_midi_ctrl_messages {
                match function.as_str() {
                    "intensity_am" => self.intensity_am = value,
                    "freq_am" => self.freq_am = value * 10.0,
                    "phase_am" => self.phase_am = value * 2.0 * PI,
                    "intensity_fm" => self.intensity_fm = value * 100.0,
                    "freq_fm" => self.freq_fm = value * 10.0,
                    "phase_fm" => self.phase_fm = value * 2.0 * PI,
                    "overdrive_gain" => self.overdrive = value * 10.0,
                    &_ => (),
                }
            }
        }
        let my_sine = SineWave::new(
            self.freq as f64,
            self.volume as f64,
            self.intensity_am as f64,
            self.freq_am as f64,
            self.phase_am as f64,
            self.intensity_fm as f64,
            self.freq_fm as f64,
            self.phase_fm as f64,
            48000.0,
            self.num_samples,
            0,
        );
        let mut _velocity: f32 = 0.0;
        if let Some(rx_note_velocity) = &self.rx_note_velocity {
            if let Ok(trigger_note_msg) = rx_note_velocity.try_recv() {
                self.freq = trigger_note_msg.freq;
                _velocity = trigger_note_msg.velocity;
            };
        };
        //        let mut effect: Option<Box<dyn Effect>> = None;
        let mut map: ParameterMap = HashMap::new();
        if self.overdrive_toggle {
            map.insert(
                "overdrive".to_string(),
                vec![
                    format!("gain {}", self.overdrive).to_string(),
                    format!("bypass {}", false).to_string(),
                ],
            );
        } else {
            map.insert(
                "overdrive".to_string(),
                vec![format!("bypass {}", true).to_string()],
            );
        }
        let effect_params: Option<ParameterMap> = Some(map);
        let msg = CtrlMsg {
            size: 1024,
            intensity_am: self.intensity_am,
            freq_am: self.freq_am,
            phase_am: self.phase_am,
            intensity_fm: self.intensity_fm,
            freq_fm: self.freq_fm,
            phase_fm: self.phase_fm,
            num_samples: self.num_samples,
            volume: self.volume,
            effect_params,
        };
        if let Some(ref x) = self.tx_ctrl {
            let _ = x.send(msg);
        }
        let msg_adsr = Adsr {
            ta: self.attack,
            td: self.decay,
            ts: self.sustain,
            tr: self.release,
        };
        if let Some(x) = &self.tx_adsr {
            let _ = x.send(msg_adsr);
        }
        let (_, values_data) = my_sine.gen_values();

        let wave_line = Line::new(PlotPoints::from_ys_f32(&values_data));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Oscillator");
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Num Samples: ");
                    ui.add(egui::DragValue::new(&mut self.num_samples).speed(100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Freq Base: ");
                    ui.add(egui::DragValue::new(&mut self.freq).speed(1.0));
                    ui.label("Volume: ");
                    ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity AM: ");
                    ui.add(egui::Slider::new(&mut self.intensity_am, 0.0..=1.0));
                    ui.label("Freq AM: ");
                    ui.add(egui::Slider::new(&mut self.freq_am, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Phase AM: ");
                    ui.add(egui::Slider::new(
                        &mut self.phase_am,
                        0.0..=std::f32::consts::TAU,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity FM: ");
                    ui.add(egui::Slider::new(&mut self.intensity_fm, 0.0..=100.0));
                    ui.label("Freq FM: ");
                    ui.add(egui::Slider::new(&mut self.freq_fm, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Phase AM: ");
                    ui.add(egui::Slider::new(
                        &mut self.phase_fm,
                        0.0..=std::f32::consts::TAU,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Overdrive: ");
                    ui.add(status_button(&mut self.overdrive_toggle));
                    ui.add(egui::Slider::new(&mut self.overdrive, 0.0..=10.0));
                });

                ui.horizontal(|ui| {
                    Plot::new("my_wave")
                        .view_aspect(2.0)
                        .data_aspect(self.num_samples as f32 / 4.0)
                        .show(ui, |plot_ui| plot_ui.line(wave_line));
                });
                ui.horizontal(|ui| {
                    ui.label("Attack: ");
                    ui.add(egui::Slider::new(&mut self.attack, 0.0..=1.0));
                    ui.label("Decay: ");
                    ui.add(egui::Slider::new(&mut self.decay, 0.0..=1.0));
                    ui.label("Sustain: ");
                    ui.add(egui::Slider::new(&mut self.sustain, 0.0..=1.0));
                    ui.label("Release: ");
                    ui.add(egui::Slider::new(&mut self.release, 0.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Length: ");
                    ui.add(egui::Slider::new(&mut self.length, 0..=192000));

                    let trigger_button = ui.button("trigger").interact(egui::Sense {
                        click: true,
                        drag: true,
                        focusable: true,
                    });
                    let triger_button_rect = trigger_button.rect;
                    ui.input(|input| {
                        if input.pointer.button_pressed(PointerButton::Primary)
                            && input.pointer.press_origin().is_some()
                            && triger_button_rect.contains(input.pointer.press_origin().unwrap())
                        {
                            if let Some(x) = &self.tx_trigger {
                                let trigger_note = TriggerNoteMsg {
                                    note_type: NoteType::NoteOn,
                                    freq: self.freq,
                                    velocity: self.velocity,
                                    length: self.length,
                                };
                                if let Err(e) = x.send(trigger_note) {
                                    println!("could send trigger_note e: {}", e);
                                };
                            }
                        } else if input.pointer.button_released(PointerButton::Primary) {
                            if let Some(x) = &self.tx_trigger {
                                let trigger_note_off = TriggerNoteMsg {
                                    note_type: NoteType::NoteOff,
                                    freq: self.freq,
                                    velocity: self.velocity,
                                    length: self.length,
                                };
                                if let Err(e) = x.send(trigger_note_off) {
                                    println!("could send trigger_note_off e: {}", e);
                                };
                            }
                        }
                    });
                    ui.label("Velocity: ");
                    ui.add(egui::Slider::new(&mut self.velocity, 0.0..=1.0));

                    if ui.button("close").clicked() {
                        if let Some(ref mut x) = self.tx_close {
                            if let Err(e) = x.try_broadcast(false) {
                                println!("could send close e: {}", e);
                            };

                            if let Some(jack_thread) = self.jack_thread.take() {
                                jack_thread.join().unwrap();
                            }
                            println!("jack_thread closed");
                            if let Some(midi_thread) = self.midi_thread.take() {
                                midi_thread.join().unwrap();
                            }
                            println!("midig_thread closed");
                            ctx.send_viewport_cmd(ViewportCommand::Close)
                        };
                    }
                })
            });
        });
    }
    fn on_exit(&mut self, _gl: Option<&Context>) {
        if let Some(ref mut tx_close) = self.tx_close {
            if let Err(e) = tx_close.try_broadcast(false) {
                println!("could not send close e: {}", e);
            };

            if let Some(jack_thread) = self.jack_thread.take() {
                jack_thread.join().unwrap();
            }
            println!("jack_thread closed");
        }
    }
}

fn repainter<MsgType>(
    ctx: egui::Context,
    rx_msg: Option<Receiver<MsgType>>,
    tx_msg: Option<Sender<MsgType>>,
) {
    if let Some(rx_msg) = rx_msg {
        loop {
            if let Ok(trigger_msg) = rx_msg.recv() {
                if let Some(ref tx_msg) = tx_msg {
                    if let Err(e) = tx_msg.send(trigger_msg) {
                        println!("could send trigger msg in repainter e: {}", e);
                    };
                }
            }
            ctx.request_repaint();
        }
    }
}
