use std::time::Duration;

use rusb::{Device, DeviceHandle, UsbContext};

const USB_STATE_FD: u8 = 0x84;

pub struct X1mk1<T: UsbContext> {
    pub device: Device<T>,
    pub handle: DeviceHandle<T>,
    pub serial_number: String,
}

struct Endpoint {
    config: u8,
    interface: u8,
    setting: u8,
    address: u8,
}

impl<T: UsbContext> X1mk1<T> {
    pub(crate) fn read(&mut self) -> rusb::Result<()> {
        println!("Reading from {}", self.serial_number);
        let endpoint = Endpoint {
            address: USB_STATE_FD,
            config: 1,
            interface: 0,
            setting: 0,
        };
        self.read_endpoint(&endpoint)?;
        Ok(())
    }

    fn read_endpoint(&mut self,
                     endpoint: &Endpoint,
    ) -> rusb::Result<()> {
        let mut buf = [0; 24];
        self.configure_endpoint(&endpoint)?;
        let timeout = Duration::from_millis(50);
        loop {
            match self.handle.read_bulk(endpoint.address, &mut buf, timeout) {
                Ok(len) => {
                    println!(" - {}: {:?}", self.serial_number, buf);
                }
                Err(e) => {
                    continue;
                }
            }
        }
        Ok(())
    }


    fn configure_endpoint(&mut self,
                          endpoint: &Endpoint,
    ) -> rusb::Result<()> {
        self.handle.set_active_configuration(endpoint.config)?;
        self.handle.claim_interface(endpoint.interface)?;
        self.handle.set_alternate_setting(endpoint.interface, endpoint.setting)?;
        Ok(())
    }
}
