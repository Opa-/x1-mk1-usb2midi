use std::sync::mpsc;

use rusb::{Context, Device, HotplugBuilder, Registration, UsbContext};

use crate::kontrol_x1mk1::X1mk1;
use crate::usb::read_device;
use crate::usb_hotplug::HotPlugHandler;
use crate::utils::get_serial_number;

mod usb_hotplug;
mod kontrol_x1mk1;
mod usb;
mod utils;

fn main() -> rusb::Result<()> {
    if rusb::has_hotplug() {
        let context = Context::new()?;
        let (tx, rx) = mpsc::channel::<Device<Context>>();

        let mut reg: Option<Registration<Context>> = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .register(&context, Box::new(HotPlugHandler { sender: tx }))?,
        );

        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(dev) => {
                        match dev.open() {
                            Ok(mut handle) => {
                                let serial_number = get_serial_number(&handle).trim().to_uppercase().to_owned();
                                let mut x1mk1 = X1mk1 { handle, serial_number };
                                x1mk1.read();
                                read_device(&mut x1mk1);
                            }
                            Err(_) => todo!(),
                        };
                    }
                    Err(_) => todo!(),
                };
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
