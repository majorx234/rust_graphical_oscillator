use crate::ctrl_msg::CtrlMsg;
use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};
use oscillator_lib::wave_gen::SineWave;

pub struct OscillatorGui {
    pub freq: f32,
    pub intensity_am: f32,
    pub freq_am: f32,
    pub phase_am: f32,
    pub intensity_fm: f32,
    pub freq_fm: f32,
    pub phase_fm: f32,
    pub num_samples: usize,
    pub tx_close: Option<std::sync::mpsc::Sender<bool>>,
    pub tx_ctrl: Option<std::sync::mpsc::Sender<CtrlMsg>>,
    pub rx_note_volume: Option<std::sync::mpsc::Receiver<(f32, f32)>>,
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
            num_samples: 48000,
            tx_close: None,
            tx_ctrl: None,
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
            Some(note_volume) => match note_volume.try_recv() {
                Ok((note, vol)) => {
                    self.freq = note;
                    volume = vol;
                }
                Err(_) => {}
            },
            None => {}
        }
        let msg = CtrlMsg {
            size: 1024,
            freq: self.freq,
            intensity_am: self.intensity_am,
            freq_am: self.freq_am,
            phase_am: self.phase_am,
            intensity_fm: self.intensity_fm,
            freq_fm: self.freq_fm,
            phase_fm: self.phase_fm,
            num_samples: self.num_samples,
        };
        match &self.tx_ctrl {
            Some(x) => {
                x.send(msg).unwrap();
            }
            None => {
                println!("No tx_ctrl\n");
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
                    if ui.button("close").clicked() {
                        match &self.tx_close {
                            Some(x) => {
                                x.send(false).unwrap();
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
