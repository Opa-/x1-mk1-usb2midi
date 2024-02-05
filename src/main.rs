use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use midir::MidiOutput;

use rusb::{Context, Device, HotplugBuilder, Registration, UsbContext};
use crate::conf::YamlConfig;

use crate::x1_process::X1mk1;
use crate::usb_hotplug::HotPlugHandler;
use crate::utils::get_serial_number;

mod x1_process;
mod usb_hotplug;
mod utils;
mod conf;
mod x1_board;

const USB_ID_VENDOR: u16 = 0x17cc;
const USB_ID_PRODUCT: u16 = 0x2305;

fn main() -> rusb::Result<()> {
    let mut file = File::open("./board.yml").expect("Failed to open YAML file");
    let mut yaml_content = String::new();
    file.read_to_string(&mut yaml_content).expect("Failed to read YAML file");

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

        thread::spawn(move || loop {
            let device = rx.recv().unwrap();
            let desc = device.device_descriptor().unwrap();
            let handle = device.open().unwrap();
            let serial_number = get_serial_number(&device).trim().to_uppercase();
            let yaml_config: YamlConfig = serde_yaml::from_str(&yaml_content).expect("Failed to parse YAML");
            thread::spawn(move || {
                let midi_out = MidiOutput::new("MIDI Kontrol X1 Mk1").unwrap();
                let mut x1mk1 = X1mk1::new(device, handle, serial_number, yaml_config.clone(), midi_out);
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
