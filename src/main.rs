use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;

use rusb::{Context, Device, HotplugBuilder, Registration, UsbContext};
use system_status_bar_macos::{Menu, MenuItem, StatusItem, sync_infinite_event_loop};

use crate::conf::YamlConfig;
use crate::usb_hotplug::HotPlugHandler;
use crate::utils::{get_serial_number, get_yaml_file};
use crate::x1_process::X1mk1;

mod x1_process;
mod usb_hotplug;
mod utils;
mod conf;
mod x1_board;

const USB_ID_VENDOR: u16 = 0x17cc;
const USB_ID_PRODUCT: u16 = 0x2305;

fn main() {
    let (sender_menu_bar, receiver_menu_bar) = mpsc::channel::<HashMap<String, bool>>();

    thread::spawn(|| {
        x1(sender_menu_bar).unwrap();
    });

    let status_item = RefCell::new(StatusItem::new("🎛️", Menu::new(vec![])));

    sync_infinite_event_loop(receiver_menu_bar, move |x1| {
        println!("Received: {:?}", x1);
        let items = x1
            .iter()
            .map(|(name, connected)| MenuItem::new(format!("{} {}", if *connected { "🟢" } else { "🔴" }, name), None, None))
            .collect();
        status_item.borrow_mut().set_menu(Menu::new(items));
    });
}

fn x1(sender_menu_bar: Sender<HashMap<String, bool>>) -> rusb::Result<()> {
    let mut file = get_yaml_file();
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

        let devices = Arc::new(Mutex::new(HashMap::new()));
        thread::spawn({
            let devices = Arc::clone(&devices);
            let sender_menu_bar = sender_menu_bar.clone();
            move || loop {
                let device = rx.recv().unwrap();
                println!("{:?}", devices.lock().unwrap());
                let handle = device.open().unwrap();
                let serial_number = get_serial_number(&device);
                let serial_number_clone = serial_number.clone();
                let yaml_config: YamlConfig = serde_yaml::from_str(&yaml_content).expect("Failed to parse YAML");
                devices.lock().unwrap().insert(serial_number_clone.clone(), true);
                sender_menu_bar.send(devices.lock().unwrap().clone()).unwrap();
                thread::spawn({
                    let devices = Arc::clone(&devices);
                    let sender_menu_bar = sender_menu_bar.clone();
                    move || {
                        let mut x1mk1 = X1mk1::new(device, handle, serial_number, yaml_config.clone());
                        loop {
                            match x1mk1.read() {
                                Ok(x) => x,
                                Err(e) => {
                                    eprintln!("Error reading from device: {:?}", e);
                                    devices.lock().unwrap().insert(serial_number_clone, false);
                                    sender_menu_bar.send(devices.lock().unwrap().clone()).unwrap();
                                    break;
                                }
                            };
                        }
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
