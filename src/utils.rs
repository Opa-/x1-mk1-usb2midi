use std::time::Duration;

use rusb::{Device, UsbContext};

pub fn get_serial_number<T: UsbContext>(dev: &Device<T>) -> String {
    let handle = dev.open().unwrap();
    let timeout = Duration::from_secs(1);
    let descriptor = handle.device().device_descriptor().unwrap();
    let languages = handle.read_languages(timeout).unwrap();
    handle
        .read_serial_number_string(languages[0], &descriptor, timeout)
        .unwrap_or_default()
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

pub fn knob_to_midi(knob: [char; 2]) -> u8 {
    let combined_value = ((knob[0] as u16) << 8) | knob[1] as u16;
    let midi_value = (combined_value as f32 / 0xFFF as f32 * 127.0).round() as u8;
    return midi_value;
}
