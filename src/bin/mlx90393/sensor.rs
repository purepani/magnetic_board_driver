#![no_std]
use core::{array, usize};

use bitflags::{bitflags, Flags};
use bitvec::prelude::*;
use defmt::debug;
use defmt::*;
use embassy_stm32::i2c::Error;
use embassy_time::Timer;
use embedded_hal::digital::v2::InputPin;
use embedded_hal_async::digital::{self, Wait};
use embedded_hal_async::i2c::I2c;
use heapless::Vec;

bitflags! {
    pub struct MagneticFieldReturnFlags: u8 {
        const X = 0b00000010;
        const Y = 0b00000100;
        const Z = 0b00001000;
        const T = 0b00000001;
    }
}

struct MemoryLocation {
    register: u8,
    position: u16,
    length: u8,
}

#[repr(u32)]
pub enum CustomerMemoryArea {
    Hallconf,
    GainSel,
    ZSeries,
    Bist,
    AnaReservedLow,
    BurstDataRate,
    BurstSel,
    TcmpEn,
    ExtTrg,
    WocDiff,
    CommMode,
    TrigInt,
    OSR,
    DigFilt,
    ResX,
    ResY,
    ResZ,

    OSR2,
    SensTcLT,
    SensTcHT,
    OffsetX,
    OffsetY,
    OffsetZ,
    WOxyThreshold,
    WOzThreshold,
}
impl CustomerMemoryArea {
    fn to_memory_location(&self) -> MemoryLocation {
        match self {
            CustomerMemoryArea::Hallconf => MemoryLocation {
                register: 0x00,
                position: 0,
                length: 4,
            },
            CustomerMemoryArea::GainSel => MemoryLocation {
                register: 0x00,
                position: 4,
                length: 3,
            },
            CustomerMemoryArea::ZSeries => MemoryLocation {
                register: 0x00,
                position: 7,
                length: 1,
            },
            CustomerMemoryArea::Bist => MemoryLocation {
                register: 0x00,
                position: 8,
                length: 1,
            },
            CustomerMemoryArea::AnaReservedLow => MemoryLocation {
                register: 0x00,
                position: 9,
                length: 7,
            },
            CustomerMemoryArea::BurstDataRate => MemoryLocation {
                register: 0x01,
                position: 0,
                length: 6,
            },
            CustomerMemoryArea::BurstSel => MemoryLocation {
                register: 0x01,
                position: 6,
                length: 4,
            },
            CustomerMemoryArea::TcmpEn => MemoryLocation {
                register: 0x01,
                position: 10,
                length: 1,
            },
            CustomerMemoryArea::ExtTrg => MemoryLocation {
                register: 0x01,
                position: 11,
                length: 1,
            },
            CustomerMemoryArea::WocDiff => MemoryLocation {
                register: 0x01,
                position: 12,
                length: 1,
            },
            CustomerMemoryArea::CommMode => MemoryLocation {
                register: 0x01,
                position: 13,
                length: 2,
            },
            CustomerMemoryArea::TrigInt => MemoryLocation {
                register: 0x01,
                position: 15,
                length: 1,
            },
            CustomerMemoryArea::OSR => MemoryLocation {
                register: 0x02,
                position: 0,
                length: 2,
            },
            CustomerMemoryArea::DigFilt => MemoryLocation {
                register: 0x02,
                position: 2,
                length: 3,
            },
            CustomerMemoryArea::ResX => MemoryLocation {
                register: 0x02,
                position: 5,
                length: 2,
            },
            CustomerMemoryArea::ResY => MemoryLocation {
                register: 0x02,
                position: 7,
                length: 2,
            },
            CustomerMemoryArea::ResZ => MemoryLocation {
                register: 0x02,
                position: 9,
                length: 2,
            },
            CustomerMemoryArea::OSR2 => MemoryLocation {
                register: 0x02,
                position: 11,
                length: 2,
            },
            CustomerMemoryArea::SensTcLT => MemoryLocation {
                register: 0x03,
                position: 0,
                length: 8,
            },
            CustomerMemoryArea::SensTcHT => MemoryLocation {
                register: 0x03,
                position: 8,
                length: 8,
            },
            CustomerMemoryArea::OffsetX => MemoryLocation {
                register: 0x04,
                position: 0,
                length: 16,
            },
            CustomerMemoryArea::OffsetY => MemoryLocation {
                register: 0x05,
                position: 0,
                length: 16,
            },
            CustomerMemoryArea::OffsetZ => MemoryLocation {
                register: 0x06,
                position: 0,
                length: 16,
            },
            CustomerMemoryArea::WOxyThreshold => MemoryLocation {
                register: 0x07,
                position: 0,
                length: 16,
            },
            CustomerMemoryArea::WOzThreshold => MemoryLocation {
                register: 0x08,
                position: 0,
                length: 16,
            },
        }
    }
}

//From datasheet
#[repr(u32)]
pub enum Command {
    SB(MagneticFieldReturnFlags) = 0b00010000,
    SW(MagneticFieldReturnFlags) = 0b00100000,
    SM(MagneticFieldReturnFlags) = 0b00110000,
    RM(MagneticFieldReturnFlags) = 0b01000000,
    RR {
        location: CustomerMemoryArea,
    } = 0b01010000,
    WR {
        D: u16,
        location: CustomerMemoryArea,
    } = 0b01100000,
    EX = 0b00001111,
    HR = 0b11010000,
    HS = 0b11100000,
    RT = 0b10000000,
}

enum CommandList {
    Single([u8; 1]),
    Double([u8; 2]),
    Quad([u8; 4]),
}

impl Command {
    fn cmds(&self) -> CommandList {
        match self {
            Command::RR { location } => {
                CommandList::Double([self.cmd1(), (location.to_memory_location().register << 2)])
            }
            Command::WR { D, location } => CommandList::Quad([
                self.cmd1(),
                D.to_le_bytes()[0],
                D.to_le_bytes()[1],
                (location.to_memory_location().register << 2),
            ]),

            _ => CommandList::Single([self.cmd1()]),
        }
    }
    fn cmd1(&self) -> u8 {
        match self {
            Command::SB(ret) => 0b00010000 + ret.bits(),
            Command::SW(ret) => 0b00100000 + ret.bits(),
            Command::SM(ret) => 0b00110000 + ret.bits(),
            Command::RM(ret) => 0b01000000 + ret.bits(),

            Command::RR { location: _ } => 0b01010000,
            Command::WR { D: _, location: _ } => 0b01100000,
            Command::EX => 0b00001111,
            Command::HR => 0b11010000,
            Command::HS => 0b11100000,
            Command::RT => 0b10000000,
        }
    }

    fn expected_number_of_return_bytes(&self) -> u8 {
        match self {
            Command::SB(_) => 1,
            Command::SW(_) => 1,
            Command::SM(_) => 1,
            Command::RM(ret) => 1 + ret.bits().count_ones() as u8,

            Command::RR { location: _ } => 3,
            Command::WR { D: _, location: _ } => 1,
            Command::EX => 1,
            Command::HR => 1,
            Command::HS => 1,
            Command::RT => 1,
        }
    }
}

bitflags! {
    struct StatusFlags: u8 {
        const burst = 0b10000000;
        const woc = 0b01000000;
        const sm = 0b00100000;
        const error = 0b00010000;
        const sed = 0b00001000;
        const rs = 0b00000100;
        const data = 0b00000011;
    }
}

#[derive(defmt::Format)]
struct Status {
    burst_mode: bool,
    woc_mode: bool,
    sm_mode: bool,
    error: bool,
    sed: bool,
    rs: bool,
    data: u8,
}

impl Status {
    fn from_u8(status: &u8) -> Self {
        let x = StatusFlags::from_bits_retain(*status);
        Status {
            burst_mode: x.contains(StatusFlags::burst),
            woc_mode: x.contains(StatusFlags::woc),
            sm_mode: x.contains(StatusFlags::sm),
            error: x.contains(StatusFlags::error),
            sed: x.contains(StatusFlags::sed),
            rs: x.contains(StatusFlags::rs),
            data: (x & StatusFlags::data).bits(),
        }
    }
}

pub struct MLX90393<T, P> {
    pub address: u8,
    pub interrupt: P,
    i2c: T,
}

impl<T: I2c, P: Wait> MLX90393<T, P> {
    pub fn new(address: u8, interrupt: P, i2c: T) -> Self {
        Self {
            address,
            interrupt,
            i2c,
        }
    }
    pub async fn reset(&mut self) {
        let status: &mut [u8; 1] = &mut [0];
        let _ = self
            .i2c
            .write_read(self.address, &[Command::EX.cmd1()], status)
            .await;
        Timer::after_millis(1).await;
        debug!("{}", Status::from_u8(&status[0]));
        let _ = self
            .i2c
            .write_read(self.address, &[Command::RT.cmd1()], status)
            .await;
        Timer::after_micros(1500).await;
        debug!("{:#010b}", status);
    }
    async fn write_u8<const N: usize>(&mut self, arr: [u8; N]) -> Result<(), T::Error> {
        for val in arr {
            self.i2c.write(self.address, &[val]).await?;
        }
        Ok(())
    }
    pub async fn run_command(&mut self, command: Command) -> &mut Self {
        let commands = command.cmds();
        let _ = match commands {
            CommandList::Single(arr) => self.write_u8(arr).await,
            CommandList::Double(arr) => self.write_u8(arr).await,
            CommandList::Quad(arr) => self.write_u8(arr).await,
        };
        //let status: &mut [u8; 1] = &mut [0; 1];
        //let _ = self.i2c.read(self.address, status).await;
        //let status_struct = Status::from_u8(status);
        //debug!("{}", status_struct);

        self
    }

    pub async fn set_sm(&mut self) {
        let flags = MagneticFieldReturnFlags::all();
        info!("Settings Mode to Single Measurement.");
        self.run_command(Command::SM(flags)).await;
        let status: &mut [u8; 1] = &mut [0; 1];
        let _ = self.i2c.read(self.address, status).await;
        let status_struct = Status::from_u8(&status[0]);
        debug!("{}", status_struct);
    }

    pub async fn read_register(&mut self, reg: CustomerMemoryArea) {
        let command = Command::RR { location: reg };
        self.run_command(command).await;

        let bytes: &mut [u8; 3] = &mut [0; 3];
        let _ = self.i2c.read(self.address, bytes).await;
        let status_struct = Status::from_u8(&bytes[0]);
        debug!("{}", status_struct);

        let data: &mut [u8] = &mut bytes[1..3];
        let _ = self.i2c.read(self.address, data).await;
        info!("{}", data)
    }

    pub async fn set_woc(&mut self) {
        let flags = MagneticFieldReturnFlags::all();
        info!("Settings Mode to Single Measurement.");
        self.run_command(Command::SW(flags)).await;
        let status: &mut [u8; 1] = &mut [0; 1];
        let _ = self.i2c.read(self.address, status).await;
        let status_struct = Status::from_u8(&status[0]);
        debug!("{}", status_struct);
    }
    pub async fn set_burst(&mut self) {
        let flags = MagneticFieldReturnFlags::all();
        info!("Settings Mode to Single Measurement.");
        self.run_command(Command::SB(flags)).await;
        let status: &mut [u8; 1] = &mut [0; 1];
        let _ = self.i2c.read(self.address, status).await;
        let status_struct = Status::from_u8(&status[0]);
        debug!("{}", status_struct);
    }

    pub async fn wait_for_measurement(&mut self) {
        let _ = self
            .run_command(Command::RM(MagneticFieldReturnFlags::all()))
            .await;
        let _ = self.interrupt.wait_for_high().await;

        let bytes: &mut [u8; 9] = &mut [0; 9];
        //let status: &mut [u8; 1] = &mut [0; 1];
        //let _ = self.i2c.read(self.address, status).await;
        //debug!("{}", status_struct);
        let _ = self.i2c.read(self.address, bytes).await;
        let status_struct = Status::from_u8(&bytes[0]);
        debug!("{}", status_struct);
        debug!("{}", bytes);
    }
}
