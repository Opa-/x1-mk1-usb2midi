use std::time::Duration;

use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;
use rusb::{Device, DeviceHandle, UsbContext};

use crate::conf::YamlConfig;
use crate::utils::{hex2bin, hex2bool, knob_to_midi};
use crate::x1_board::{ButtonType, X1mk1Board};

const USB_WRITE_FD: u8 = 0x01;
const USB_UNLOCK_FD: u8 = 0x81;
const USB_READ_FD: u8 = 0x84;
const MIDI_MSG_FIRST_BYTE: u8 = 0xB0;

pub struct X1mk1<T: UsbContext> {
    pub device: Device<T>,
    pub handle: DeviceHandle<T>,
    pub serial_number: String,
    midi_conn_out: MidiOutputConnection,
    board: X1mk1Board,
}

struct Endpoint {
    config: u8,
    interface: u8,
    setting: u8,
    address: u8,
}

impl<T: UsbContext> X1mk1<T> {
    pub fn new(device: Device<T>, handle: DeviceHandle<T>, serial_number: String, yaml_config: YamlConfig, midi_out: MidiOutput) -> Self {
        let mut midi_conn_out = midi_out.create_virtual(serial_number.as_str()).unwrap();
        let board = X1mk1Board::from_yaml(&yaml_config);

        Self {
            device,
            handle,
            serial_number,
            midi_conn_out,
            board,
        }
    }

    pub(crate) fn read(&mut self) -> rusb::Result<()> {
        println!("Reading from {}", self.serial_number);
        let endpoint = Endpoint {
            address: USB_READ_FD,
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
        let mut unlock_buf = [0; 1]; // Need to read 1 byte after write to unlock the device.
        // self.write_state(&mut unlock_buf);
        // self.down(&mut useless_buf);
        loop {
            match self.handle.read_bulk(endpoint.address, &mut buf, timeout) {
                Ok(len) => {
                    // println!(" - {}: {:?}", self.serial_number, buf);
                    // println!("{:?}", self);
                    self.read_state(buf);
                    // self.midi_toggle();
                    // self.midi_hold();
                    // self.midi_knobs();
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

    // fn midi_toggle(&mut self) {
    //     // Have to compare to old state for toggle buttons such as "Play/Pause"
    //     if !self.state_before.button[10][0] && self.state_current.button[10][0] {
    //         let _ = self.midi_conn_out.send(&[0xB0, 0x00, 127]);
    //     }
    // }
    //
    // fn midi_hold(&mut self) {
    //     if self.state_current.button[8][0] {
    //         let _ = self.midi_conn_out.send(&[0xB0, 0x02, 127]);
    //     } else {
    //         let _ = self.midi_conn_out.send(&[0xB0, 0x02, 0]);
    //     }
    // }
    //
    // fn midi_knobs(&mut self) {
    //     self.midi_conn_out.send(&[0xB0, 0x01, knob_to_midi(self.state_current.knob[0][0]) as u8]);
    // }

    fn read_state(&mut self, buf: [u8; 24]) {
        let mut binbyte: [[bool; 8]; 5] = [[false; 8]; 5];

        hex2bool(buf[1], &mut binbyte[0]);
        hex2bool(buf[2], &mut binbyte[1]);
        hex2bool(buf[3], &mut binbyte[2]);
        hex2bool(buf[4], &mut binbyte[3]);
        hex2bool(buf[5], &mut binbyte[4]);

        for (ctrl_name, button_type) in &mut self.board.buttons {
            match button_type {
                ButtonType::Toggle(ref mut button)
                | ButtonType::Hold(ref mut button) => {
                    button.curr = binbyte[button.read_i as usize][button.read_j as usize];
                    if (button.curr != button.prev) {
                        println!("{} changed from {} to {}", ctrl_name, button.prev, button.curr);
                    }
                    button.prev = button.curr;
                }
                ButtonType::Knob(ref mut knob) => {
                    knob.curr = knob_to_midi(buf[knob.read_i as usize], buf[knob.read_j as usize]);
                    if (knob.curr != knob.prev) {
                        println!("{} changed from {} to {}", ctrl_name, knob.prev, knob.curr);
                    }
                    knob.prev = knob.curr;
                }
                ButtonType::Encoder(ref mut encoder) => {
                    let mut binnum = [0; 8];
                    hex2bin(buf[encoder.read_i as usize], &mut binnum);
                    match encoder.read_pos {
                        's' => {
                            encoder.curr = binnum[0] + binnum[1] * 2 + binnum[2] * 4 + binnum[3] * 8;
                        },
                        'e' => {
                            encoder.curr = binnum[4] + binnum[5] * 2 + binnum[6] * 4 + binnum[7] * 8;
                        },
                        _ => panic!("Invalid read_pos"),
                    }
                    if (encoder.curr != encoder.prev) {
                        println!("{} changed from {} to {}", ctrl_name, encoder.prev, encoder.curr);
                    }
                    encoder.prev = encoder.curr;
                }
            }
        }
    }

    fn write_state(&self, unlock_buf: &mut [u8; 1]) {
        let mut leds = [0x05; 32];
        leds[0] = 0x0C;
        leds[31] = 0;
        self.handle.write_bulk(USB_WRITE_FD, &leds, Duration::from_millis(50)).unwrap();
        self.handle.read_bulk(USB_UNLOCK_FD, unlock_buf, Duration::from_millis(50)).unwrap();
    }

    fn down(&self, unlock_buf: &mut [u8; 1]) {
        let mut leds = [0x00; 32];
        leds[0] = 0x0C;
        self.handle.write_bulk(USB_WRITE_FD, &leds, Duration::from_millis(50)).unwrap();
        self.handle.read_bulk(USB_UNLOCK_FD, unlock_buf, Duration::from_millis(50)).unwrap();
    }
}
