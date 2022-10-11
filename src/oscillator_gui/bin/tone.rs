use crate::trigger_note_msg::NoteType;

#[derive(Debug)]
pub struct Tone {
    pub playing: bool,
    pub note_type: NoteType,
    pub freq: f32,
    pub volume: f32,
    pub start_pose: u32,
    pub envelope: Option<Vec<f32>>,
}
