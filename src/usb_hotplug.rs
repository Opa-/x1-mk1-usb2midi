use std::sync::mpsc;

use rusb::{Device, UsbContext};

pub struct HotPlugHandler<T: UsbContext> {
    pub sender: mpsc::Sender<Device<T>>,
}

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler<T> {
    fn device_arrived(&mut self, device: Device<T>) {
        match device.device_descriptor() {
            Ok(_) => {
                println!("🟢 Device arrived {:?}", device);
                self.sender.send(device).unwrap();
            }
            Err(err) => eprintln!("Error getting device descriptor: {:?}", err),
        };
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("🟠 Device left {:?}", device);
    }
}

impl<T: UsbContext> Drop for HotPlugHandler<T> {
    fn drop(&mut self) {
        println!("🔴 HotPlugHandler dropped");
    }
}
