use rusb::{Device, DeviceDescriptor, DeviceHandle, Direction, TransferType, UsbContext};
use std::time::Duration;
use crate::USB_STATE_FD;

pub fn read_device<T: UsbContext>(
    device: &Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> rusb::Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(10);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Languages: {:?}", languages);

    if !languages.is_empty() {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
    }

    let endpoint = Endpoint {
        address: USB_STATE_FD,
        config: 1,
        interface: 0,
        setting: 0,
    };
    // read_endpoint(handle, &endpoint, TransferType::Bulk);
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
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
    transfer_type: TransferType,
) {
    // println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.interface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.interface).ok();
            true
        }
        _ => false,
    };

    // println!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut buf = [0; 24];
            let timeout = Duration::from_millis(50);
            loop {
                match transfer_type {
                    TransferType::Bulk => match handle.read_bulk(endpoint.address, &mut buf, timeout) {
                        Ok(len) => println!(" - read: {:?}", buf),
                        Err(err) => println!(" - read: {:?}", buf),
                    },
                    _ => (),
                }
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.interface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> rusb::Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.interface)?;
    handle.set_alternate_setting(endpoint.interface, endpoint.setting)?;
    Ok(())
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    interface: u8,
    setting: u8,
    address: u8,
}
