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
const LED_DIM: u8 = 0x05;
const LED_BRIGHT: u8 = 0x7F;
const MIDI_MSG_FIRST_BYTE: u8 = 0xB0;

pub struct X1mk1<T: UsbContext> {
    pub device: Device<T>,
    pub handle: DeviceHandle<T>,
    pub serial_number: String,
    midi_conn_out: MidiOutputConnection,
    board: X1mk1Board,
    usb_buffer: [u8; 24],
    usb_timeout: Duration,
    led: [u8; 33],
    led_updated: bool,
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
        let mut leds = [0x05; 33];
        leds[0] = 0x0C;
        leds[32] = 0;
        let usb_buffer = [0; 24];

        Self {
            device,
            handle,
            serial_number,
            midi_conn_out,
            board,
            usb_buffer,
            usb_timeout: Duration::from_millis(50),
            led: leds,
            led_updated: false,
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
        self.update_leds();
        loop {
            self.led_updated = false;
            match self.handle.read_bulk(endpoint.address, &mut self.usb_buffer, self.usb_timeout) {
                Ok(len) => {
                    // println!("read  {:?}", buf);
                    if len != self.usb_buffer.len() {
                        // rusb crate consider partially read data as ok but we do not.
                        continue;
                    }
                    self.read_state(self.usb_buffer.clone());
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
            if self.led_updated {
                self.update_leds();
            }
        }
    }

    fn configure_endpoint(&mut self, endpoint: &Endpoint) -> rusb::Result<()> {
        self.handle.set_active_configuration(endpoint.config)?;
        self.handle.claim_interface(endpoint.interface)?;
        self.handle.set_alternate_setting(endpoint.interface, endpoint.setting)?;
        Ok(())
    }

    fn read_state(&mut self, buf: [u8; 24]) {
        let mut binbyte: [[bool; 8]; 5] = [[false; 8]; 5];

        hex2bool(buf[1], &mut binbyte[0]);
        hex2bool(buf[2], &mut binbyte[1]);
        hex2bool(buf[3], &mut binbyte[2]);
        hex2bool(buf[4], &mut binbyte[3]);
        hex2bool(buf[5], &mut binbyte[4]);

        for (ctrl_name, button_type) in &mut self.board.buttons {
            match button_type {
                ButtonType::Toggle(ref mut button) => {
                    button.curr = binbyte[button.read_i as usize][button.read_j as usize];
                    if (button.curr != button.prev) {
                        // println!("{} changed from {} to {}", ctrl_name, button.prev, button.curr);
                        if (button.curr) {
                            let l = self.led[button.write_idx as usize];
                            self.led[button.write_idx as usize] = if l == LED_DIM { LED_BRIGHT } else { LED_DIM };
                            self.led_updated = true;
                            let _ = self.midi_conn_out.send(&[MIDI_MSG_FIRST_BYTE, button.midi_ctrl_ch, 127]);
                        }
                    }
                    button.prev = button.curr;
                }
                ButtonType::Hold(ref mut button) => {
                    button.curr = binbyte[button.read_i as usize][button.read_j as usize];
                    if (button.curr != button.prev) {
                        // println!("{} changed from {} to {}", ctrl_name, button.prev, button.curr);
                        if (button.curr) {
                            self.led[button.write_idx as usize] = LED_BRIGHT;
                            let _ = self.midi_conn_out.send(&[MIDI_MSG_FIRST_BYTE, button.midi_ctrl_ch, 127]);
                        } else {
                            self.led[button.write_idx as usize] = LED_DIM;
                            let _ = self.midi_conn_out.send(&[MIDI_MSG_FIRST_BYTE, button.midi_ctrl_ch, 0]);
                        }
                        self.led_updated = true;
                    }
                    button.prev = button.curr;
                }
                ButtonType::Knob(ref mut knob) => {
                    knob.curr = knob_to_midi(buf[knob.read_i as usize], buf[knob.read_j as usize]);
                    if (knob.curr != knob.prev) {
                        let _ = self.midi_conn_out.send(&[MIDI_MSG_FIRST_BYTE, knob.midi_ctrl_ch, knob.curr]);
                    }
                    knob.prev = knob.curr;
                }
                ButtonType::Encoder(ref mut encoder) => {
                    let mut binnum = [0; 8];
                    hex2bin(buf[encoder.read_i as usize], &mut binnum);
                    match encoder.read_pos {
                        's' => {
                            encoder.curr = binnum[0] + binnum[1] * 2 + binnum[2] * 4 + binnum[3] * 8;
                        }
                        'e' => {
                            encoder.curr = binnum[4] + binnum[5] * 2 + binnum[6] * 4 + binnum[7] * 8;
                        }
                        _ => panic!("Invalid read_pos"),
                    }
                    if encoder.curr != encoder.prev {
                        // Clockwise init
                        let mut velocity = 1;
                        if (encoder.prev == 15 && encoder.curr == 0) || (encoder.prev == 0 && encoder.curr == 15) {
                            // Full rotation special case
                            velocity = if encoder.prev == 15 { 1 } else { 127 };
                        } else if encoder.curr < encoder.prev {
                            // Clockwise
                            velocity = 127;
                        }
                        let _ = self.midi_conn_out.send(&[MIDI_MSG_FIRST_BYTE, encoder.midi_ctrl_ch, velocity]);
                    }
                    encoder.prev = encoder.curr;
                }
            }
        }
    }

    fn update_leds(&self) {
        self.handle.write_bulk(USB_WRITE_FD, &self.led, self.usb_timeout).unwrap();
        match self.handle.read_bulk(USB_UNLOCK_FD, &mut [0; 1], self.usb_timeout) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading from device: {:?}", e);
            }
        };
    }
}
