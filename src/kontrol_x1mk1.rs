use std::fmt;
use std::io::Write;
use std::time::Duration;

use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;
use rusb::{Device, DeviceHandle, UsbContext};

use crate::utils::{hex2bin, hex2bool, knob_to_midi};

const USB_STATE_FD: u8 = 0x84;

pub struct X1mk1<T: UsbContext> {
    pub device: Device<T>,
    pub handle: DeviceHandle<T>,
    pub serial_number: String,
    midi_conn_out: MidiOutputConnection,
    button: [[bool; 4]; 11],
    knob: [[[char; 2]; 2]; 6],
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
            button: [[false; 4]; 11],
            knob: [[['\0'; 2]; 2]; 6],
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
        loop {
            match self.handle.read_bulk(endpoint.address, &mut buf, timeout) {
                Ok(len) => {
                    println!(" - {}: {:?}", self.serial_number, buf);
                    println!("{}", self);
                    self.write_state(buf);
                    if self.button[10][0] == true {
                        self.play_note(1);
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

    fn write_state(&mut self, buf: [u8; 24]) {
        let mut binbyte: [[bool; 8]; 5] = [[false; 8]; 5];

        hex2bool(buf[1], &mut binbyte[0]);
        hex2bool(buf[2], &mut binbyte[1]);
        hex2bool(buf[3], &mut binbyte[2]);
        hex2bool(buf[4], &mut binbyte[3]);
        hex2bool(buf[5], &mut binbyte[4]);

        self.button[0][0] = binbyte[3][4];
        self.button[0][1] = binbyte[4][0];
        self.button[1][0] = binbyte[3][5];
        self.button[1][1] = binbyte[4][1];
        self.button[2][0] = binbyte[3][6];
        self.button[2][1] = binbyte[4][2];
        self.button[3][0] = binbyte[3][7];
        self.button[3][1] = binbyte[4][3];

        self.button[4][0] = binbyte[3][0];
        self.button[4][1] = binbyte[4][4];
        self.button[4][2] = binbyte[3][1];
        self.button[5][0] = binbyte[1][1];
        self.button[5][1] = binbyte[1][0];
        self.button[5][2] = binbyte[4][6];
        self.button[5][3] = binbyte[4][5];
        self.button[6][0] = binbyte[3][2];
        self.button[6][1] = binbyte[4][7];
        self.button[6][2] = binbyte[3][3];

        self.button[7][0] = binbyte[2][4];
        self.button[7][1] = binbyte[0][3];
        self.button[7][2] = binbyte[1][4];
        self.button[7][3] = binbyte[2][3];
        self.button[8][0] = binbyte[0][2];
        self.button[8][1] = binbyte[2][5];
        self.button[8][2] = binbyte[2][2];
        self.button[8][3] = binbyte[1][5];
        self.button[9][0] = binbyte[0][1];
        self.button[9][1] = binbyte[2][6];
        self.button[9][2] = binbyte[2][1];
        self.button[9][3] = binbyte[1][6];
        self.button[10][0] = binbyte[0][0];
        self.button[10][1] = binbyte[2][7];
        self.button[10][2] = binbyte[2][0];
        self.button[10][3] = binbyte[1][7];

        self.set_knob_with_char(0, 0, buf[16], buf[17]);
        self.set_knob_with_char(0, 1, buf[12], buf[13]);
        self.set_knob_with_char(1, 0, buf[20], buf[21]);
        self.set_knob_with_char(1, 1, buf[10], buf[11]);
        self.set_knob_with_char(2, 0, buf[22], buf[23]);
        self.set_knob_with_char(2, 1, buf[8], buf[9]);
        self.set_knob_with_char(3, 0, buf[18], buf[19]);
        self.set_knob_with_char(3, 1, buf[14], buf[15]);

        self.set_encoder_with_char(4, 0, buf[6], 's');
        self.set_encoder_with_char(4, 1, buf[6], 'e');
        self.set_encoder_with_char(5, 0, buf[7], 's');
        self.set_encoder_with_char(5, 1, buf[7], 'e');
    }

    fn set_knob_with_char(&mut self, m: usize, n: usize, c1: u8, c2: u8) {
        // Little endian.
        self.knob[m][n][0] = char::from(c1);
        self.knob[m][n][1] = char::from(c2);
    }

    fn set_encoder_with_char(&mut self, m: usize, n: usize, c: u8, pos: char) {
        let mut binnum = [0; 8];
        hex2bin(c, &mut binnum);

        match pos {
            's' => {
                self.knob[m][n][0] = char::from(binnum[0] + binnum[1] * 2 + binnum[2] * 4 + binnum[3] * 8);
            }
            'e' => {
                self.knob[m][n][0] = char::from(binnum[4] + binnum[5] * 2 + binnum[6] * 4 + binnum[7] * 8);
            }
            _ => {}
        }
    }
}

impl<T: UsbContext> fmt::Display for X1mk1<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "=================\n");
        write!(f, "{} {} {} {}\n", self.button[0][0], knob_to_midi(self.knob[0][0]), knob_to_midi(self.knob[0][1]), self.button[0][1]);
        write!(f, "{} {} {} {}\n", self.button[1][0], knob_to_midi(self.knob[1][0]), knob_to_midi(self.knob[1][1]), self.button[1][1]);
        write!(f, "{} {} {} {}\n", self.button[2][0], knob_to_midi(self.knob[2][0]), knob_to_midi(self.knob[2][1]), self.button[2][1]);
        write!(f, "{} {} {} {}\n", self.button[3][0], knob_to_midi(self.knob[3][0]), knob_to_midi(self.knob[3][1]), self.button[3][1]);
        write!(f, "\n");
        write!(f, "{} {}\n", knob_to_midi(self.knob[4][0]), knob_to_midi(self.knob[4][1]));
        write!(f, "{} {} {}\n", self.button[4][0], self.button[4][1], self.button[4][2]);
        write!(f, "{} {} {} {}\n", self.button[5][0], self.button[5][1], self.button[5][2], self.button[5][3]);
        write!(f, "{} {}\n", knob_to_midi(self.knob[5][0]), knob_to_midi(self.knob[5][1]));
        write!(f, "{} {} {}\n", self.button[6][0], self.button[6][1], self.button[6][2]);
        write!(f, "\n");
        write!(f, "{} {} {} {}\n", self.button[7][0], self.button[7][1], self.button[7][2], self.button[7][3]);
        write!(f, "{} {} {} {}\n", self.button[8][0], self.button[8][1], self.button[8][2], self.button[8][3]);
        write!(f, "{} {} {} {}\n", self.button[9][0], self.button[9][1], self.button[9][2], self.button[9][3]);
        write!(f, "{} {} {} {}\n", self.button[10][0], self.button[10][1], self.button[10][2], self.button[10][3]);
        write!(f, "=================");
        Ok(())
    }
}
