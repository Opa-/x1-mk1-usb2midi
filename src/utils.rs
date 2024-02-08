use std::fs::File;
use std::time::Duration;

use rusb::{Device, UsbContext};

pub fn get_serial_number<T: UsbContext>(dev: &Device<T>) -> String {
    let handle = dev.open().unwrap();
    let timeout = Duration::from_secs(1);
    let descriptor = handle.device().device_descriptor().unwrap();
    let languages = handle.read_languages(timeout).unwrap();
    handle
        .read_serial_number_string(languages[0], &descriptor, timeout)
        .unwrap_or_default().trim().to_uppercase()
}

pub fn hex2bool(hex: u8, bin: &mut [bool; 8]) {
    for i in 0..8 {
        bin[i] = ((hex >> i) & 1) != 0;
    }
}

pub fn hex2bin(hex: u8, bin: &mut [u8; 8]) {
    for i in 0..8 {
        bin[i] = (hex >> i) & 1;
    }
}

pub fn knob_to_midi(i: u8, j: u8) -> u8 {
    let combined_value = ((i as u16) << 8) | j as u16;
    
    (combined_value as f32 / 0xFFF as f32 * 127.0).round() as u8
}

pub fn get_yaml_file() -> std::fs::File {
    let yaml_path = get_yaml_path();
    File::open(yaml_path).unwrap_or_else(|_| File::open("board.yml").expect("Failed to open board.yml"))
}

fn get_yaml_path() -> String {
    let mut resources_dir = std::env::current_exe().expect("Failed to get current executable path");
    resources_dir.pop(); // Remove the executable name
    resources_dir.pop(); // Remove macOS directory
    resources_dir.push("Resources");
    resources_dir.push("board.yml");
    return resources_dir.to_str().unwrap().to_string();
}
