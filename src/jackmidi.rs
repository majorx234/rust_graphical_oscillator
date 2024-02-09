use crate::util::*;
use serde::{Deserialize, Serialize};
use std::convert::From;

const MAX_MIDI: usize = 3;
type MidiId = u16;
type Note = u8;
type Intensity = u8;

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub enum MidiMsgAdvanced {
    MidiEmpty,
    MidiNoteOnOff(MidiId, MidiId, bool, Note, Intensity),
    MidiControlIdValue(MidiId, u16),
    MidiControl2IdsValue(MidiId, MidiId, u16),
}

impl MidiMsgAdvanced {
    pub fn get_id(&self) -> u16 {
        match self {
            Self::MidiEmpty => 0,
            Self::MidiNoteOnOff(id0, _, _, _, _) => *id0,
            Self::MidiControlIdValue(id, _) => *id,
            Self::MidiControl2IdsValue(id0, _, _) => *id0,
        }
    }

    pub fn get_value(&self) -> u16 {
        match self {
            Self::MidiEmpty => 0,
            Self::MidiNoteOnOff(_, _, _, _, value) => *value as u16,
            Self::MidiControlIdValue(_, value) => *value,
            Self::MidiControl2IdsValue(_, _, value) => *value,
        }
    }

    pub fn get_norm_value(&self) -> f32 {
        match self {
            Self::MidiEmpty => 0.0,
            Self::MidiNoteOnOff(_, _, _, _, value) => *value as f32 / 127.0,
            Self::MidiControlIdValue(_, value) => *value as f32 / 127.0,
            Self::MidiControl2IdsValue(_, _, value) => *value as f32 / 16383.0,
        }
    }
}

impl MidiMsgAdvanced {
    pub fn from_midi_msg_cc(midi_msg: MidiMsgControlChange) -> Self {
        let midi_id = midi_msg.get_id();
        let midi_value = 128 * midi_msg.get_value();
        MidiMsgAdvanced::MidiControlIdValue(midi_id, midi_value)
    }

    pub fn from_midi_msg_cc2ids(
        midi_msg0: MidiMsgControlChange,
        midi_msg1: MidiMsgControlChange,
    ) -> Self {
        let midi_id0 = midi_msg0.get_id();
        let midi_id1 = midi_msg1.get_id();

        let midi_value = 128 * midi_msg0.get_value() + midi_msg1.get_value();

        MidiMsgAdvanced::MidiControl2IdsValue(midi_id0, midi_id1, midi_value)
    }

    pub fn from_midi_msgs_note_on(midi_msg: MidiMsgNoteOn) -> Self {
        let midi_id_on = midi_msg.get_id();
        let midi_value = true;
        let midi_id_off = midi_id_on - 0x1000;
        let midi_note = midi_msg.get_data()[1];
        let midi_intensity = midi_msg.get_data()[2];

        MidiMsgAdvanced::MidiNoteOnOff(
            midi_id_on,
            midi_id_off,
            midi_value,
            midi_note,
            midi_intensity,
        )
    }

    pub fn from_midi_msgs_note_off(midi_msg: MidiMsgNoteOff) -> Self {
        let midi_id_on = midi_msg.get_id() + 0x1000;
        let midi_value = false;
        let midi_id_off = midi_msg.get_id();
        let midi_note = midi_msg.get_data()[1];
        let midi_intensity = midi_msg.get_data()[2];

        MidiMsgAdvanced::MidiNoteOnOff(
            midi_id_on,
            midi_id_off,
            midi_value,
            midi_note,
            midi_intensity,
        )
    }

    pub fn from_current_and_last_opt_midi_msgs(
        (current_midi_msg, last_opt_midi_msg): (
            Box<dyn MidiMsgBase>,
            &mut Option<Box<dyn MidiMsgBase>>,
        ),
    ) -> Option<Self> {
        let mut id = current_midi_msg.get_id();
        let last_last_midi_msg = last_opt_midi_msg.take();
        let midi_msg_value = current_midi_msg.get_value();
        let midi_msgs_data = current_midi_msg.get_data();
        let midi_msg_type = current_midi_msg.type_of().to_string();
        let midi_msg_timestamp = current_midi_msg.get_time();
        let mut id_value_time_diff_to_last_msg = None;
        if let Some(last_last_midi_msg) = last_last_midi_msg {
            let time_diff = midi_msg_timestamp.abs_diff(last_last_midi_msg.get_time());
            let last_id = last_last_midi_msg.get_id();
            let last_value = last_last_midi_msg.get_value();
            if time_diff < 10 && id > last_id {
                id_value_time_diff_to_last_msg = Some((last_id, last_value, time_diff));
            }
        }
        *last_opt_midi_msg = Some(current_midi_msg);
        match midi_msg_type.as_str() {
            "MidiMsgControlChange" => {
                if let Some((last_id, last_value, _time_diff)) = id_value_time_diff_to_last_msg {
                    Some(MidiMsgAdvanced::MidiControl2IdsValue(
                        last_id,
                        id,
                        midi_msg_value + last_value * 128,
                    ))
                } else {
                    Some(MidiMsgAdvanced::MidiControlIdValue(id, midi_msg_value))
                }
            }
            "MidiMsgNoteOn" => Some(MidiMsgAdvanced::MidiNoteOnOff(
                id,
                id - 0x1000,
                true,
                midi_msgs_data[1],
                midi_msgs_data[2],
            )),
            "MidiMsgNoteOff" => {
                id += 0x1000;
                Some(MidiMsgAdvanced::MidiNoteOnOff(
                    id,
                    id - 0x1000,
                    false,
                    midi_msgs_data[1],
                    midi_msgs_data[2],
                ))
            }
            "MidiMsgPitchBend" => Some(MidiMsgAdvanced::MidiControl2IdsValue(
                id,
                id,
                midi_msg_value,
            )),
            _ => None,
        }
    }
}

impl std::fmt::Display for MidiMsgAdvanced {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MidiEmpty => write!(f, "MidiEmpty"),
            Self::MidiNoteOnOff(id0, id1, value, note, intensity) => {
                write!(
                    f,
                    "NoteOnOff({}, {}, {}, {}, {})",
                    id0, id1, *value, note, intensity
                )
            }
            Self::MidiControlIdValue(id, value) => {
                write!(f, "MidiControlIdValue({}, {})", id, value)
            }
            Self::MidiControl2IdsValue(id0, id1, value) => {
                write!(f, "MidiControl2IdsValue({}, {}, {})", id0, id1, value)
            }
        }
    }
}

impl std::fmt::Debug for MidiMsgAdvanced {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MidiEmpty => write!(f, "MidiEmpty"),
            Self::MidiNoteOnOff(id0, id1, value, note, intensity) => {
                write!(
                    f,
                    "NoteOnOff({}, {}, {}, {}, {})",
                    id0, id1, *value, note, intensity
                )
            }
            Self::MidiControlIdValue(id, value) => {
                write!(f, "MidiControlIdValue({}, {})", id, value)
            }
            Self::MidiControl2IdsValue(id0, id1, value) => {
                write!(f, "MidiControl2IdsValue({}, {}, {})", id0, id1, value)
            }
        }
    }
}

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
