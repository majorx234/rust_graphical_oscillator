use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};
use oscillator_lib::wave_gen::SineWave;

struct OszilatorGui {
    size: usize,
    wave_data: std::vec::Vec<f32>,
}

impl Default for OszilatorGui {
    fn default() -> Self {
        Self {
            size: 0,
            wave_data: Vec::new(),
        }
    }
}

impl eframe::App for OszilatorGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let wave = (0..self.size).map(|i| {
            let x = i as f64;
            Value::new(x, self.wave_data[i] as f64)
        });

        let wave_line = Line::new(Values::from_values_iter(wave));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plot");
            ui.horizontal(|ui| {
                Plot::new("my_wave")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| plot_ui.line(wave_line));
            })
        });
    }
}

fn main() {
    let my_sine = SineWave::new(44, 48000);
    let (values_size, values_data) = my_sine.get_values();
    let plot_app = OszilatorGui {
        wave_data: values_data.clone(),
        size: values_size,
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native("Plot App", options, Box::new(|_cc| Box::new(plot_app)));
}
