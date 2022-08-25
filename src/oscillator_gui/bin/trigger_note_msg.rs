pub enum NoteType {
    NoteOn,
    NoteOff,
}

pub struct TriggerNoteMsg {
    pub note_type: NoteType,
    pub freq: f32,
    pub velocity: f32,
}
