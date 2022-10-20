#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum NoteType {
    NoteOn,
    NoteOff,
}

#[derive(Clone, Copy)]
pub struct TriggerNoteMsg {
    pub note_type: NoteType,
    pub freq: f32,
    pub velocity: f32,
    pub length: usize,
}
