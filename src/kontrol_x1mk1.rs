use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use midir::os::unix::VirtualOutput;
use rusb::{Device, DeviceHandle, UsbContext};

const USB_STATE_FD: u8 = 0x84;

pub struct X1mk1<T: UsbContext> {
    pub device: Device<T>,
    pub handle: DeviceHandle<T>,
    pub serial_number: String,
    midi_conn_out: MidiOutputConnection,
}

struct Endpoint {
    config: u8,
    interface: u8,
    setting: u8,
    address: u8,
}

impl<T: UsbContext> X1mk1<T> {
    pub fn new(device: Device<T>, handle: DeviceHandle<T>, serial_number: String, midi_out: MidiOutput) -> Self {
        let mut midi_conn_out = midi_out.create_virtual(serial_number.as_str()).unwrap();

        Self {
            device,
            handle,
            serial_number,
            midi_conn_out,
        }
    }

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

    fn read_endpoint(&mut self, endpoint: &Endpoint) -> rusb::Result<()> {
        self.configure_endpoint(&endpoint)?;

        let timeout = Duration::from_millis(50);
        let mut buf = [0; 24];
        let mut b = false;
        loop {
            match self.handle.read_bulk(endpoint.address, &mut buf, timeout) {
                Ok(len) => {
                    println!(" - {}: {:?}", self.serial_number, buf);
                    if buf[1] == 1 {
                        if !b {
                            self.play_note(1);
                        }
                        b = true;
                    } else {
                        b = false;
                    }
                }
                Err(e) => {
                    if e == rusb::Error::Timeout {
                        // Weird timeout occurring when all knobs are at 0 position and no button is pressed.
                        // We do not want to break because there's no need to call configure_endpoint again.
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    fn configure_endpoint(&mut self, endpoint: &Endpoint) -> rusb::Result<()> {
        self.handle.set_active_configuration(endpoint.config)?;
        self.handle.claim_interface(endpoint.interface)?;
        self.handle.set_alternate_setting(endpoint.interface, endpoint.setting)?;
        Ok(())
    }

    fn play_note(&mut self, note: u8) {
        const NOTE_ON_MSG: u8 = 0x90;
        const NOTE_OFF_MSG: u8 = 0x80;
        const VELOCITY: u8 = 0x64;
        let _ = self.midi_conn_out.send(&[NOTE_ON_MSG, note, 0x90]);
    }
}
