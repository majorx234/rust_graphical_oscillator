use crate::util::*;
use serde::{Deserialize, Serialize};
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

pub trait MidiMsgBase: Send + std::fmt::Display {
    fn type_of(&self) -> &str;
    fn get_data(&self) -> Vec<u8>;
    fn get_id(&self) -> u16;
    fn get_value(&self) -> u16;
    fn get_time(&self) -> u64;
}

//a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
pub struct MidiMsgGeneric {
    pub len: usize,
    pub data: [u8; MAX_MIDI],
    pub time: u64,
}

impl MidiMsgBase for MidiMsgGeneric {
    fn type_of(&self) -> &str {
        "MidiMsgGeneric"
    }
    fn get_data(&self) -> Vec<u8> {
        self.data.into_iter().collect()
    }
    fn get_id(&self) -> u16 {
        u16::max_value()
    }
    fn get_value(&self) -> u16 {
        0
    }
    fn get_time(&self) -> u64 {
        self.time
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

pub struct MidiMsgControlChange {
    pub channel: u8,
    pub control: u8,
    pub value: u8,
    pub time: u64,
}

impl MidiMsgBase for MidiMsgControlChange {
    fn type_of(&self) -> &str {
        "MidiMsgControlChange"
    }
    fn get_data(&self) -> Vec<u8> {
        vec![0xB0 + self.channel, self.control, self.value]
    }
    fn get_id(&self) -> u16 {
        0xB000 + ((self.channel as u16) << 8) + self.control as u16
    }
    fn get_value(&self) -> u16 {
        self.value as u16
    }
    fn get_time(&self) -> u64 {
        self.time
    }
}

impl std::fmt::Debug for MidiMsgControlChange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiControlChange: time: {}, len: 3, channel: {}, control: {}, value: {}",
            self.time, self.channel, self.control, self.value,
        )
    }
}

impl std::fmt::Display for MidiMsgControlChange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiControlChange: time: {}, len: 3, channel: {}, control: {}, value: {}",
            self.time, self.channel, self.control, self.value,
        )
    }
}

pub struct MidiMsgNoteOn {
    pub channel: u8,
    pub key: u8,
    pub velocity: u8,
    pub time: u64,
}

impl MidiMsgBase for MidiMsgNoteOn {
    fn type_of(&self) -> &str {
        "MidiMsgNoteOn"
    }
    fn get_data(&self) -> Vec<u8> {
        vec![0x90 + self.channel, self.key, self.velocity]
    }
    fn get_id(&self) -> u16 {
        0x9000 + ((self.channel as u16) << 8) + self.key as u16
    }
    fn get_value(&self) -> u16 {
        self.velocity as u16
    }
    fn get_time(&self) -> u64 {
        self.time
    }
}

impl std::fmt::Debug for MidiMsgNoteOn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiNoteOn: time: {}, len: 3, channel: {}, key: {}, velocity: {}",
            self.time, self.channel, self.key, self.velocity,
        )
    }
}

impl std::fmt::Display for MidiMsgNoteOn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiNoteOn: time: {}, len: 3, channel: {}, key: {}, velocity: {}",
            self.time, self.channel, self.key, self.velocity,
        )
    }
}

pub struct MidiMsgNoteOff {
    pub channel: u8,
    pub key: u8,
    pub velocity: u8,
    pub time: u64,
}

impl MidiMsgBase for MidiMsgNoteOff {
    fn type_of(&self) -> &str {
        "MidiMsgNoteOff"
    }
    fn get_data(&self) -> Vec<u8> {
        vec![0x80 + self.channel, self.key, self.velocity]
    }
    fn get_id(&self) -> u16 {
        0x8000 + ((self.channel as u16) << 8) + self.key as u16
    }
    fn get_value(&self) -> u16 {
        self.velocity as u16
    }
    fn get_time(&self) -> u64 {
        self.time
    }
}

impl std::fmt::Debug for MidiMsgNoteOff {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiNoteOff: time: {}, len: 3, channel: {}, key: {}, velocity: {}",
            self.time, self.channel, self.key, self.velocity,
        )
    }
}

impl std::fmt::Display for MidiMsgNoteOff {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiNoteOff: time: {}, len: 3, channel: {}, key: {}, velocity: {}",
            self.time, self.channel, self.key, self.velocity,
        )
    }
}

pub struct MidiMsgPitchBend {
    pub channel: u8,
    pub value: u16,
    pub time: u64,
}

impl MidiMsgBase for MidiMsgPitchBend {
    fn type_of(&self) -> &str {
        "MidiMsgPitchBend"
    }
    fn get_data(&self) -> Vec<u8> {
        let (msb_value, lsb_value) = u14_to_msb_lsb(self.value);
        vec![0xE0 + self.channel, lsb_value, msb_value]
    }
    fn get_id(&self) -> u16 {
        0xE000 + ((self.channel as u16) << 8)
    }
    fn get_value(&self) -> u16 {
        self.value
    }
    fn get_time(&self) -> u64 {
        self.time
    }
}

impl std::fmt::Debug for MidiMsgPitchBend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiMsgPitchBend: time: {}, len: 3, channel: {}, value: {}",
            self.time, self.channel, self.value,
        )
    }
}

impl std::fmt::Display for MidiMsgPitchBend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiMsgPitchBend: time: {}, len: 3, channel: {}, value: {}",
            self.time, self.channel, self.value,
        )
    }
}

impl From<jack::RawMidi<'_>> for Box<dyn MidiMsgBase> {
    fn from(midi: jack::RawMidi<'_>) -> Box<dyn MidiMsgBase> {
        let midi_time = midi.time as u64 + jack::get_time();
        let len = std::cmp::min(MAX_MIDI, midi.bytes.len());
        let (status, channel) = from_status_byte(midi.bytes[0]);
        if status == 0x08 {
            // NoteOff
            Box::new(MidiMsgNoteOff {
                channel,
                key: mask7(midi.bytes[1]),
                velocity: mask7(midi.bytes[2]),
                time: midi_time,
            })
        } else if status == 0x09 {
            // NoteOn
            Box::new(MidiMsgNoteOn {
                channel,
                key: mask7(midi.bytes[1]),
                velocity: mask7(midi.bytes[2]),
                time: midi_time,
            })
        } else if status == 0x0b {
            // MidiCC
            Box::new(MidiMsgControlChange {
                channel,
                control: mask7(midi.bytes[1]),
                value: mask7(midi.bytes[2]),
                time: midi_time,
            })
        } else if status == 0x0e {
            // MidiPitchBend
            Box::new(MidiMsgPitchBend {
                channel,
                value: msb_lsb_to_u14(mask7(midi.bytes[2]), mask7(midi.bytes[1])),
                time: midi_time,
            })
        } else {
            let mut data = [0; MAX_MIDI];
            data[..len].copy_from_slice(&midi.bytes[..len]);
            Box::new(MidiMsgGeneric {
                len,
                data,
                time: midi_time,
            })
        }
    }
}

impl From<MidiMsgGeneric> for Box<dyn MidiMsgBase> {
    fn from(midi: MidiMsgGeneric) -> Box<dyn MidiMsgBase> {
        let midi_time = midi.time;
        let len = midi.len;
        let (status, channel) = from_status_byte(midi.data[0]);
        if status == 0x08 {
            // NoteOff
            Box::new(MidiMsgNoteOff {
                channel,
                key: mask7(midi.data[1]),
                velocity: mask7(midi.data[2]),
                time: midi_time,
            })
        } else if status == 0x09 {
            // NoteOn
            Box::new(MidiMsgNoteOn {
                channel,
                key: mask7(midi.data[1]),
                velocity: mask7(midi.data[2]),
                time: midi_time,
            })
        } else if status == 0x0b {
            // MidiCC
            Box::new(MidiMsgControlChange {
                channel,
                control: mask7(midi.data[1]),
                value: mask7(midi.data[2]),
                time: midi_time,
            })
        } else if status == 0x0e {
            // MidiPitchBend
            Box::new(MidiMsgPitchBend {
                channel,
                value: msb_lsb_to_u14(mask7(midi.data[2]), mask7(midi.data[1])),
                time: midi_time,
            })
        } else {
            let mut data = [0; MAX_MIDI];
            data[..len].copy_from_slice(&midi.data[..len]);
            Box::new(MidiMsgGeneric {
                len,
                data,
                time: midi_time,
            })
        }
    }
}
