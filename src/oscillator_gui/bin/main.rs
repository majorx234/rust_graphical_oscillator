use oscillator_lib::wave_gen::SineWave;

fn main() {
    let my_sine = SineWave::new(440, 48000);
    my_sine.print();
}
