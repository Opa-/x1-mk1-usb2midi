use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum YamlButtonType {
    Toggle,
    Hold,
    Knob,
    Encoder,
    Hotcue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct YamlButton {
    pub name: String,
    #[serde(rename = "type")]
    pub button_type: YamlButtonType,
    pub read_i: u8,
    pub read_j: Option<u8>,
    pub read_pos: Option<char>,
    pub write_idx: Option<u8>,
    pub midi_ctrl_ch: u8,
    pub hotcue_ignore: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct YamlConfig {
    pub buttons: Vec<YamlButton>,
}
