use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::result::Result;
use std::{fmt, fs::File};

use crate::jackmidi::MidiMsgAdvanced;

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct MidiFunction {
    name: String,
    // ToDo: invert, log,linear, scaling..
}

impl MidiFunction {
    pub fn new(name: String) -> Self {
        MidiFunction { name }
    }
    pub fn get_name(&self) -> String {
        self.name.as_str().to_string()
    }
}

impl fmt::Display for MidiFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MidiFunctionFile {
    pub midi_functions: Vec<MidiFunction>,
}

pub fn parse_json_file_to_midi_functions(
    file_path_str: &String,
) -> Result<MidiFunctionFile, String> {
    let mut file_content =
        File::open(file_path_str).map_err(|err| format!("Could not read the json file {}", err))?;
    let mut contents = String::new();
    file_content
        .read_to_string(&mut contents)
        .map_err(|err| format!("Could not read file to string {}", err))?;
    let module: MidiFunctionFile = serde_json::from_str(contents.as_str())
        .map_err(|err| format!("error in json deserialize {}", err))?;
    Ok(module)
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MidiFunctionWithElementsFile {
    pub midi_functions: HashMap<String, Vec<u16>>,
}

pub fn parse_json_file_to_midi_functions_with_elements_ids(
    file_path_str: &String,
) -> Result<HashMap<String, Vec<u16>>, String> {
    let mut file_content =
        File::open(file_path_str).map_err(|err| format!("Could not read the json file {}", err))?;
    let mut contents = String::new();
    file_content
        .read_to_string(&mut contents)
        .map_err(|err| format!("Could not read file to string {}", err))?;
    let map: HashMap<String, Vec<u16>> = serde_json::from_str(contents.as_str()).unwrap();
    Ok(map)
}

pub fn parse_json_file_to_midi_functions_with_midi_msgs_advanced(
    file_path_str: &String,
) -> Result<HashMap<String, Vec<MidiMsgAdvanced>>, String> {
    let mut file_content =
        File::open(file_path_str).map_err(|err| format!("Could not read the json file {}", err))?;
    let mut contents = String::new();
    file_content
        .read_to_string(&mut contents)
        .map_err(|err| format!("Could not read file to string {}", err))?;
    let map: HashMap<String, Vec<MidiMsgAdvanced>> =
        serde_json::from_str(contents.as_str()).unwrap();
    Ok(map)
}

pub fn reverse_map_midi_functions2midi_advanced_msgs(
    midi_functions_with_midi_advanced_msgs: HashMap<String, Vec<MidiMsgAdvanced>>,
) -> HashMap<MidiMsgAdvanced, Vec<String>> {
    let mut midi_advanced_msgs2midi_functions: HashMap<MidiMsgAdvanced, Vec<String>> =
        HashMap::new();
    for (key, value_vec) in midi_functions_with_midi_advanced_msgs {
        let key_insert = key.clone();
        for value in value_vec {
            if let Some(ref mut midi_function_vec) =
                midi_advanced_msgs2midi_functions.get_mut(&value)
            {
                midi_function_vec.push(key_insert.clone());
            } else {
                midi_advanced_msgs2midi_functions.insert(value, vec![key_insert.clone()]);
            }
        }
    }
    midi_advanced_msgs2midi_functions
}
