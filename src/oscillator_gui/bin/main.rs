use oscillator_lib::SineWave;

fn main() {
    let my_sine = SineWave::new(440, 48000);
    my_sine.print();
}
