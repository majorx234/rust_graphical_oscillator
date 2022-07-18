use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};
use oscillator_lib::wave_gen::SineWave;
use std::sync::mpsc;

pub struct OscillatorGui {
    pub freq: f32,
    pub intensity_am: f32,
    pub freq_am: f32,
    pub intensity_fm: f32,
    pub freq_fm: f32,
    pub num_samples: usize,
    pub tx: Option<std::sync::mpsc::Sender<bool>>,
}

impl Default for OscillatorGui {
    fn default() -> Self {
        Self {
            freq: 44.0,
            intensity_am: 1.0,
            freq_am: 0.0,
            intensity_fm: 1.0,
            freq_fm: 0.0,
            num_samples: 48000,
            tx: None,
        }
    }
}

impl eframe::App for OscillatorGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let my_sine = SineWave::new(
            self.freq,
            self.intensity_am,
            self.freq_am,
            self.intensity_fm,
            self.freq_fm,
            48000.0,
            self.num_samples,
            0,
        );
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
                    ui.label("Intensity FM: ");
                    ui.add(egui::Slider::new(&mut self.intensity_fm, 0.0..=100.0));
                    ui.label("Freq FM: ");
                    ui.add(egui::Slider::new(&mut self.freq_fm, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    Plot::new("my_wave")
                        .view_aspect(2.0)
                        .show(ui, |plot_ui| plot_ui.line(wave_line));
                });
                ui.horizontal(|ui| {
                    if ui.button("close").clicked() {
                        match &self.tx {
                            Some(x) => {
                                x.send(false).unwrap();
                            }
                            None => {
                                println!("No tx\n");
                            }
                        }
                    }
                })
            });
        });
    }
}
