use std::sync::mpsc;
use std::thread;

use rusb::{Context, Device, HotplugBuilder, Registration, UsbContext};

use crate::kontrol_x1mk1::X1mk1;
use crate::usb_hotplug::HotPlugHandler;
use crate::utils::get_serial_number;

mod usb_hotplug;
mod kontrol_x1mk1;
mod utils;


const USB_ID_VENDOR: u16 = 0x17cc;
const USB_ID_PRODUCT: u16 = 0x2305;


fn main() -> rusb::Result<()> {
    if rusb::has_hotplug() {
        let context = Context::new()?;
        let (tx, rx) = mpsc::channel::<Device<Context>>();

        let mut reg: Option<Registration<Context>> = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .vendor_id(USB_ID_VENDOR)
                .product_id(USB_ID_PRODUCT)
                .register(&context, Box::new(HotPlugHandler { sender: tx }))?,
        );

        thread::spawn(move || {
            loop {
                let device = rx.recv().unwrap();
                let desc = device.device_descriptor().unwrap();
                let handle = device.open().unwrap();
                let serial_number = get_serial_number(&device).trim().to_uppercase();
                thread::spawn(move || {
                    let mut x1mk1 = X1mk1 { device, handle, serial_number };
                    loop {
                        match x1mk1.read() {
                            Ok(x) => x,
                            Err(e) => {
                                eprintln!("Error reading from device: {:?}", e);
                                break;
                            }
                        };
                    }
                });
            }
        });

        loop {
            match context.handle_events(None) {
                Ok(x) => x,
                Err(_) => {
                    if let Some(reg) = reg.take() {
                        context.unregister_callback(reg);
                    }
                }
            };
        }
    } else {
        eprintln!("libusb compiled without hotplug support");
    }
    Ok(())
}
