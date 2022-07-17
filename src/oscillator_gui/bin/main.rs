use eframe::egui;
use eframe::egui::plot::{Line, Plot, Value, Values};
extern crate jack;
use oscillator_lib::wave_gen::SineWave;
use std::sync::mpsc;
use std::{thread, time::Duration};

struct OszilatorGui {
    size: usize,
    freq: f32,
    intensity_am: f32,
    freq_am: f32,
    intensity_fm: f32,
    freq_fm: f32,
    num_samples: usize,
    tx: Option<std::sync::mpsc::Sender<bool>>,
}

impl Default for OszilatorGui {
    fn default() -> Self {
        Self {
            size: 0,
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

impl eframe::App for OszilatorGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let my_sine = SineWave::new(
            self.freq,
            self.intensity_am,
            self.freq_am,
            self.intensity_fm,
            self.freq_fm,
            48000.0,
            self.num_samples,
        );
        let (values_size, values_data) = my_sine.get_values();
        self.size = values_size;
        let wave = (0..self.size).map(|i| {
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

fn main() {
    let (tx, rx) = mpsc::channel();
    let audio_thread = start_audio_thread(rx);
    let plot_app = OszilatorGui {
        size: 0,
        freq: 44.0,
        intensity_am: 1.0,
        freq_am: 0.0,
        intensity_fm: 1.0,
        freq_fm: 0.0,
        num_samples: 48000,
        tx: Some(tx),
    };
    let options = eframe::NativeOptions::default();
    eframe::run_native("Oscillator", options, Box::new(|_cc| Box::new(plot_app)));
    audio_thread.join().unwrap();
}

fn start_audio_thread(rx: std::sync::mpsc::Receiver<bool>) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let (client, _status) =
            jack::Client::new("graphical oscillator", jack::ClientOptions::NO_START_SERVER)
                .unwrap();

        // register ports
        let mut out_a = client
            .register_port("gosci_out_l", jack::AudioOut::default())
            .unwrap();
        let mut out_b = client
            .register_port("gosci_out_r", jack::AudioOut::default())
            .unwrap();

        // sinewave generator
        // let sine_wave_generator

        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let out_a_p = out_a.as_mut_slice(ps);
            let out_b_p = out_b.as_mut_slice(ps);

            // Use the overdrive to process samples
            //sine_wave_generator.process_samples(out_a_p, out_b_p);

            jack::Control::Continue
        };
        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        let mut run: bool = true;
        while run {
            thread::sleep(Duration::from_millis(100));
            run = rx.recv().unwrap();
            println!("running: {}", run);
        }
        active_client.deactivate().unwrap();
    })
}
