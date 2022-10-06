use crate::adsr::Adsr;
use crate::ctrl_msg::CtrlMsg;
use crate::trigger_note_msg::{self, NoteType, TriggerNoteMsg};
use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};
use oscillator_lib::wave_gen::SineWave;
use std::{thread, time};

pub struct OscillatorGui {
    pub freq: f32,
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
    pub tx_close: Option<crossbeam_channel::Sender<bool>>,
    pub tx_ctrl: Option<std::sync::mpsc::Sender<CtrlMsg>>,
    pub tx_adsr: Option<std::sync::mpsc::Sender<Adsr>>,
    pub tx_trigger: Option<std::sync::mpsc::Sender<TriggerNoteMsg>>,
    pub rx_note_volume: Option<std::sync::mpsc::Receiver<TriggerNoteMsg>>,
}

impl Default for OscillatorGui {
    fn default() -> Self {
        Self {
            freq: 44.0,
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
            tx_close: None,
            tx_ctrl: None,
            tx_adsr: None,
            tx_trigger: None,
            rx_note_volume: None,
        }
    }
}

impl eframe::App for OscillatorGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let my_sine = SineWave::new(
            self.freq,
            self.intensity_am,
            self.freq_am,
            self.phase_am,
            self.intensity_fm,
            self.freq_fm,
            self.phase_fm,
            48000.0,
            self.num_samples,
            0,
        );
        let mut volume: f32 = 0.0;
        match &self.rx_note_volume {
            Some(rx_note_volume) => match rx_note_volume.try_recv() {
                Ok(trigger_note_msg) => {
                    self.freq = trigger_note_msg.freq;
                    volume = trigger_note_msg.velocity;
                }
                Err(_) => {}
            },
            None => {}
        }
        let msg = CtrlMsg {
            size: 1024,
            intensity_am: self.intensity_am,
            freq_am: self.freq_am,
            phase_am: self.phase_am,
            intensity_fm: self.intensity_fm,
            freq_fm: self.freq_fm,
            phase_fm: self.phase_fm,
            num_samples: self.num_samples,
        };
        match &self.tx_ctrl {
            Some(x) => x.send(msg).unwrap(),
            None => {
                println!("No tx_ctrl\n");
            }
        }
        let msg_adsr = Adsr {
            ta: self.attack,
            td: self.decay,
            ts: self.sustain,
            tr: self.release,
        };
        match &self.tx_adsr {
            Some(x) => x.send(msg_adsr).unwrap(),

            None => {
                println!("No tx_adsr\n");
            }
        }
        let (values_size, values_data) = my_sine.gen_values();
        let wave = (0..values_size).map(|i| {
            let x = i as f64;
            Value::new(x, values_data[i] as f64)
        });

        let wave_line = Line::new(Values::from_values_iter(wave));

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
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity AM: ");
                    ui.add(egui::Slider::new(&mut self.intensity_am, 0.0..=1.0));
                    ui.label("Freq AM: ");
                    ui.add(egui::Slider::new(&mut self.freq_am, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Phase AM: ");
                    ui.add(egui::Slider::new(&mut self.phase_am, 0.0..=6.283));
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity FM: ");
                    ui.add(egui::Slider::new(&mut self.intensity_fm, 0.0..=100.0));
                    ui.label("Freq FM: ");
                    ui.add(egui::Slider::new(&mut self.freq_fm, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Phase AM: ");
                    ui.add(egui::Slider::new(&mut self.phase_fm, 0.0..=6.283));
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
                    ui.label("length: ");
                    ui.add(egui::Slider::new(&mut self.length, 0..=192000));

                    let trigger_button = ui.button("trigger").interact(egui::Sense {
                        click: true,
                        drag: true,
                        focusable: true,
                    });
                    if trigger_button.drag_started() {
                        match &self.tx_trigger {
                            Some(x) => {
                                let trigger_note = TriggerNoteMsg {
                                    note_type: NoteType::NoteOn,
                                    freq: self.freq,
                                    velocity: 0.0,
                                    length: self.length,
                                };
                                x.send(trigger_note).unwrap();
                            }
                            None => (),
                        }
                    } else {
                        if trigger_button.drag_released() {
                            match &self.tx_trigger {
                                Some(x) => {
                                    let trigger_note_off = TriggerNoteMsg {
                                        note_type: NoteType::NoteOff,
                                        freq: self.freq,
                                        velocity: 0.0,
                                        length: self.length,
                                    };
                                    x.send(trigger_note_off).unwrap();
                                }
                                None => (),
                            }
                        }
                    }
                    if ui.button("close").clicked() {
                        match &self.tx_close {
                            Some(x) => {
                                x.send(false).unwrap();
                                _frame.quit();
                            }
                            None => {
                                println!("No tx_close\n");
                            }
                        }
                    }
                })
            });
        });
    }
}
