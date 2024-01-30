use std::convert::From;

const MAX_MIDI: usize = 3;

//a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
pub struct MidiMsg {
    pub len: usize,
    pub data: [u8; MAX_MIDI],
    pub time: jack::Frames,
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

impl std::fmt::Debug for MidiMsgGeneric {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiGeneric: time: {}, len: {}, data: {:?}",
            self.time,
            self.len,
            &self.data[..self.len]
        )
    }
}

//a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
pub struct MidiMsgGeneric {
    pub len: usize,
    pub data: [u8; MAX_MIDI],
    pub time: u64,
}

impl std::fmt::Display for MidiMsgGeneric {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiGeneric: time: {}, len: {}, data: {:?}",
            self.time,
            self.len,
            &self.data[..self.len]
        )
    }
}

impl From<jack::RawMidi<'_>> for MidiMsgGeneric {
    fn from(midi: jack::RawMidi<'_>) -> MidiMsgGeneric {
        let mut data: [u8; MAX_MIDI] = [0, 0, 0];
        data[..MAX_MIDI].copy_from_slice(&midi.bytes[..MAX_MIDI]);
        MidiMsgGeneric {
            len: MAX_MIDI,
            data,
            time: midi.time as u64 + jack::get_time(),
        }
    }
}
