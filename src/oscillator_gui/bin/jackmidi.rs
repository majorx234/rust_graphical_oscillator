use std::convert::From;
use std::io;

const MAX_MIDI: usize = 3;

//a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
pub struct MidiMsg {
    len: usize,
    data: [u8; MAX_MIDI],
    time: jack::Frames,
}

impl From<jack::RawMidi<'_>> for MidiMsg {
    fn from(midi: jack::RawMidi<'_>) -> Self {
        let len = std::cmp::min(MAX_MIDI, midi.bytes.len());
        let mut data = [0; MAX_MIDI];
        data[..len].copy_from_slice(&midi.bytes[..len]);
        MidiMsg {
            len,
            data,
            time: midi.time,
        }
    }
}

impl std::fmt::Debug for MidiMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Midi {{ time: {}, len: {}, data: {:?} }}",
            self.time,
            self.len,
            &self.data[..self.len]
        )
    }
}
