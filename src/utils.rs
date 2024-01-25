use std::time::Duration;

use rusb::{DeviceHandle, UsbContext};

fn hex2bin(hexnum: u8, binnum: &mut [u8; 8]) {
    // Taken from < https://stackoverflow.com/questions/4892579/how-to-convert-a-char-to-binary >.
    for i in 0..8 {
        binnum[i] = (hexnum >> i) & 1;
    }
}

pub fn get_serial_number<T: UsbContext>(handle: &DeviceHandle<T>) -> String {
    let timeout = Duration::from_secs(1);
    let descriptor = handle.device().device_descriptor().unwrap();
    let languages = handle.read_languages(timeout).unwrap();
    handle.read_serial_number_string(languages[0], &descriptor, timeout).unwrap_or_default()
}
