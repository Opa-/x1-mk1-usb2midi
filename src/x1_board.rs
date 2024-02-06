use std::collections::HashMap;
use crate::conf::{YamlButtonType, YamlConfig};

pub struct Button {
    pub curr: bool,
    pub prev: bool,
    pub read_i: u8,
    pub read_j: u8,
    pub write_idx: u8,
}

pub struct Knob {
    pub curr: u8,
    pub prev: u8,
    pub read_i: u8,
    pub read_j: u8,
}

pub struct Encoder {
    pub curr: u8,
    pub prev: u8,
    pub read_pos: char,
    pub read_i: u8,
}

pub enum ButtonType {
    Toggle(Button),
    Hold(Button),
    Knob(Knob),
    Encoder(Encoder),
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
                        write_idx: yaml_button.write_idx.unwrap(),
                    };
                    ButtonType::Toggle(button)
                }
                YamlButtonType::Hold => {
                    let button = Button {
                        curr: false,
                        prev: false,
                        read_i: yaml_button.read_i,
                        read_j: yaml_button.read_j.unwrap(),
                        write_idx: yaml_button.write_idx.unwrap(),
                    };
                    ButtonType::Hold(button)
                }
                YamlButtonType::Knob => {
                    let knob = Knob {
                        curr: 0,
                        prev: 0,
                        read_i: yaml_button.read_i,
                        read_j: yaml_button.read_j.unwrap(),
                    };
                    ButtonType::Knob(knob)
                }
                YamlButtonType::Encoder => {
                    let encoder = Encoder {
                        curr: 0,
                        prev: 0,
                        read_pos: yaml_button.read_pos.unwrap(),
                        read_i: yaml_button.read_i,
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