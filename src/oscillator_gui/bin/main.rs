use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};
use oscillator_lib::wave_gen::SineWave;

struct OszilatorGui {
    size: usize,
    wave_data: std::vec::Vec<f32>,
    freq: f32,
    freq_am: f32,
    freq_fm: f32,
}

impl Default for OszilatorGui {
    fn default() -> Self {
        Self {
            size: 0,
            wave_data: Vec::new(),
            freq: 44.0,
            freq_am: 0.0,
            freq_fm: 0.0,
        }
    }
}

impl eframe::App for OszilatorGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let my_sine = SineWave::new(self.freq, self.freq_am, self.freq_fm, 48000.0, 48000);
        let (values_size, values_data) = my_sine.get_values();
        self.size = values_size;
        self.wave_data = values_data.clone();
        let wave = (0..self.size).map(|i| {
            let x = i as f64;
            Value::new(x, self.wave_data[i] as f64)
        });

        let wave_line = Line::new(Values::from_values_iter(wave));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Oscillator");
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Freq Base: ");
                    ui.add(egui::DragValue::new(&mut self.freq).speed(1.0));
                    ui.label("Freq AM: ");
                    ui.add(egui::Slider::new(&mut self.freq_am, 0.0..=10.0));
                    ui.label("Freq FM: ");
                    ui.add(egui::Slider::new(&mut self.freq_fm, 0.0..=10.0));
                });
                ui.horizontal(|ui| {
                    Plot::new("my_wave")
                        .view_aspect(2.0)
                        .show(ui, |plot_ui| plot_ui.line(wave_line));
                });
            });
        });
    }
}

fn main() {
    let plot_app = OszilatorGui {
        wave_data: Vec::new(),
        size: 0,
        freq: 44.0,
        freq_am: 0.0,
        freq_fm: 0.0,
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native("Oscillator", options, Box::new(|_cc| Box::new(plot_app)));
}
