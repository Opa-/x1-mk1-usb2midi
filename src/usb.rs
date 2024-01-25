use std::time::Duration;

use rusb::{Device, DeviceDescriptor, DeviceHandle, Direction, TransferType, UsbContext};
use crate::kontrol_x1mk1::X1mk1;
use crate::utils::get_serial_number;

const USB_STATE_FD: u8 = 0x84;

pub fn read_device<T: UsbContext>(device: &mut X1mk1<T>) -> rusb::Result<()> {
    device.handle.reset()?;

    let timeout = Duration::from_secs(10);
    let languages = device.handle.read_languages(timeout)?;
    let device_desc = device.handle.device().device_descriptor()?;

    println!("Active configuration: {}", device.handle.active_configuration()?);
    println!("Languages: {:?}", languages);

    if !languages.is_empty() {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            device.handle
                .read_manufacturer_string(language, &device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            device.handle
                .read_product_string(language, &device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            device.handle
                .read_serial_number_string(language, &device_desc, timeout)
                .ok()
        );
    }

    let endpoint = Endpoint {
        address: USB_STATE_FD,
        config: 1,
        interface: 0,
        setting: 0,
    };
    read_endpoint(device, &endpoint, TransferType::Bulk);
    Ok(())
}

fn find_readable_endpoint<T: UsbContext>(
    device: &Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    println!("Looking for readable endpoint for {:?}", transfer_type);
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    println!("Trying endpoint: 0x{:x}", endpoint_desc.address());
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            interface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn read_endpoint<T: UsbContext>(
    device: &mut X1mk1<T>,
    endpoint: &Endpoint,
    transfer_type: TransferType,
) {
    // println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match device.handle.kernel_driver_active(endpoint.interface) {
        Ok(true) => {
            device.handle.detach_kernel_driver(endpoint.interface).ok();
            true
        }
        _ => false,
    };

    // println!(" - kernel driver? {}", has_kernel_driver);

    let serial_number = get_serial_number(&device.handle).trim().to_uppercase().to_owned();
    match configure_endpoint(device, &endpoint) {
        Ok(_) => {
            let mut buf = [0; 24];
            let timeout = Duration::from_millis(50);
            loop {
                match transfer_type {
                    TransferType::Bulk => match device.handle.read_bulk(endpoint.address, &mut buf, timeout) {
                        Ok(len) => println!(" - {serial_number}: {:?}", buf),
                        Err(err) => println!(" - {serial_number}: {:?}", buf),
                    },
                    _ => (),
                }
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        device.handle.attach_kernel_driver(endpoint.interface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    device: &mut X1mk1<T>,
    endpoint: &Endpoint,
) -> rusb::Result<()> {
    device.handle.set_active_configuration(endpoint.config)?;
    device.handle.claim_interface(endpoint.interface)?;
    device.handle.set_alternate_setting(endpoint.interface, endpoint.setting)?;
    Ok(())
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    interface: u8,
    setting: u8,
    address: u8,
}
