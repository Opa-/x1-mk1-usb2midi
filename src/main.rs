use std::sync::mpsc;

use rusb::{Context, Device, HotplugBuilder, Registration, UsbContext};

use crate::kontrol_x1mk1::X1mk1;
use crate::usb_hotplug::HotPlugHandler;
use crate::utils::get_serial_number;

mod usb_hotplug;
mod kontrol_x1mk1;
mod usb;
mod utils;

const USB_ID_VENDOR: u16 = 0x17cc;
const USB_ID_PRODUCT: u16 = 0x2305;
const USB_STATE_FD: u8 = 0x84;

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
                            Ok(handle) => {
                                let serial_number = get_serial_number(&handle).trim().to_uppercase().to_owned();
                                let x1mk1 = X1mk1 { handle, serial_number };
                                x1mk1.read()
                            }
                            Err(_) => todo!(),
                        };
                    }
                    Err(_) => todo!(),
                };
            }
        });

        loop {
            context.handle_events(None).unwrap();
            if let Some(reg) = reg.take() {
                context.unregister_callback(reg);
            }
        }
    } else {
        eprintln!("libusb compiled without hotplug support");
    }
    Ok(())
}
