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
