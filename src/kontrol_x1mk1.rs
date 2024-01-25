use rusb::{DeviceHandle, UsbContext};

pub struct X1mk1<T: UsbContext> {
    pub handle: DeviceHandle<T>,
    pub serial_number: String,
}

impl<T: UsbContext> X1mk1<T> {
    pub(crate) fn read(&self) {
        println!("Reading device {}", self.serial_number);
    }
}
