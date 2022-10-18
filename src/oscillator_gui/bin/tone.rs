use crate::adsr::Adsr;
use crate::jackaudio::SineWaveGenerator;
use crate::trigger_note_msg::NoteType;

#[derive(Debug)]
pub struct Tone {
    pub playing: bool,
    pub length: usize,
    pub note_type: NoteType,
    pub freq: f32,
    pub volume: f32,
    pub start_pose: u32,
    pub adsr_envelope: Adsr,
    pub envelope: Option<Vec<f32>>,
    pub last_sustain_value_a: f32,
    pub last_sustain_value_b: f32,
    pub sine_wave_generator: SineWaveGenerator,
}
