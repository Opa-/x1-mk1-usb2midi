use std::sync::mpsc;

use rusb::{Device, UsbContext};

use crate::{USB_ID_PRODUCT, USB_ID_VENDOR};

pub struct HotPlugHandler<T: UsbContext> {
    pub sender: mpsc::Sender<Device<T>>,
}

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler<T> {
    fn device_arrived(&mut self, device: Device<T>) {
        match device.device_descriptor() {
            Ok(descriptor) => {
                if (descriptor.vendor_id() == USB_ID_VENDOR && descriptor.product_id() == USB_ID_PRODUCT) {
                    self.sender.send(device).unwrap();
                }
            }
            Err(_) => todo!(),
        };
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("device left {:?}", device);
    }
}

impl<T: UsbContext> Drop for HotPlugHandler<T> {
    fn drop(&mut self) {
        println!("HotPlugHandler dropped");
    }
}
