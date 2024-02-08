use std::collections::HashMap;

use crate::conf::{YamlButtonType, YamlConfig};

pub struct Button {
    pub curr: bool,
    pub prev: bool,
    pub read_i: u8,
    pub read_j: u8,
    pub write_idx: u8,
    pub midi_ctrl_ch: u8,
    pub hotcue_ignore: bool,
}

pub struct Knob {
    pub curr: u8,
    pub prev: u8,
    pub read_i: u8,
    pub read_j: u8,
    pub midi_ctrl_ch: u8,
}

pub struct Encoder {
    pub curr: u8,
    pub prev: u8,
    pub read_pos: char,
    pub read_i: u8,
    pub midi_ctrl_ch: u8,
}

pub enum ButtonType {
    Toggle(Button),
    Hold(Button),
    Knob(Knob),
    Encoder(Encoder),
    Hotcue(Button)
}

pub struct X1mk1Board {
    pub(crate) buttons: HashMap<String, ButtonType>,
}

impl X1mk1Board {
    pub(crate) fn from_yaml(yaml_config: &YamlConfig) -> Self {
        let mut buttons: HashMap<String, ButtonType> = HashMap::new();
        for yaml_button in &yaml_config.buttons {
            let button_type = match yaml_button.button_type {
                YamlButtonType::Toggle => {
                    let button = Button {
                        curr: false,
                        prev: false,
                        read_i: yaml_button.read_i,
                        read_j: yaml_button.read_j.unwrap(),
                        write_idx: yaml_button.write_idx.unwrap_or(0),
                        midi_ctrl_ch: yaml_button.midi_ctrl_ch,
                        hotcue_ignore: yaml_button.hotcue_ignore.unwrap_or(false)
                    };
                    ButtonType::Toggle(button)
                }
                YamlButtonType::Hold => {
                    let button = Button {
                        curr: false,
                        prev: false,
                        read_i: yaml_button.read_i,
                        read_j: yaml_button.read_j.unwrap(),
                        write_idx: yaml_button.write_idx.unwrap_or(0),
                        midi_ctrl_ch: yaml_button.midi_ctrl_ch,
                        hotcue_ignore: yaml_button.hotcue_ignore.unwrap_or(false)
                    };
                    ButtonType::Hold(button)
                }
                YamlButtonType::Hotcue => {
                    let button = Button {
                        curr: false,
                        prev: false,
                        read_i: yaml_button.read_i,
                        read_j: yaml_button.read_j.unwrap(),
                        write_idx: yaml_button.write_idx.unwrap_or(0),
                        midi_ctrl_ch: yaml_button.midi_ctrl_ch,
                        hotcue_ignore: yaml_button.hotcue_ignore.unwrap_or(false)
                    };
                    ButtonType::Hotcue(button)
                }
                YamlButtonType::Knob => {
                    let knob = Knob {
                        curr: 0,
                        prev: 0,
                        read_i: yaml_button.read_i,
                        read_j: yaml_button.read_j.unwrap(),
                        midi_ctrl_ch: yaml_button.midi_ctrl_ch,
                    };
                    ButtonType::Knob(knob)
                }
                YamlButtonType::Encoder => {
                    let encoder = Encoder {
                        curr: 0,
                        prev: 0,
                        read_pos: yaml_button.read_pos.unwrap(),
                        read_i: yaml_button.read_i,
                        midi_ctrl_ch: yaml_button.midi_ctrl_ch,
                    };
                    ButtonType::Encoder(encoder)
                }
            };
            buttons.insert(yaml_button.name.clone(), button_type);
        }
        X1mk1Board {
            buttons
        }
    }
}
