use bitflags::{bitflags, Flags};
use bitmatch::bitmatch;
use defmt::Format;

pub struct Register<const R: u8> {
    data: [u8; 2],
}

impl<const R: u8> Register<R> {
    pub fn new(data: [u8; 2]) -> Self {
        Self { data }
    }
}

impl Register<0x00> {
    fn flags(&self) -> RegisterOneFlags {
        RegisterOneFlags::from_bits_retain(u16::from_be_bytes(self.data))
    }

    pub fn zseries(&self) -> ZSeries {
        match self.flags().contains(RegisterOneFlags::ZSeries) {
            true => ZSeries::Enabled,
            false => ZSeries::Disabled,
        }
    }

    pub fn bist(&self) -> Bist {
        match self.flags().contains(RegisterOneFlags::Bist) {
            true => Bist::Enabled,
            false => Bist::Disabled,
        }
    }

    pub fn hall_conf(&self) -> Option<HallConf> {
        HallConf::from_u8_slice(&self.data)
    }

    pub fn gain(&self) -> Gain {
        Gain::from_u8_slice(&self.data)
    }
}

pub struct BurstSel {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub temp: bool,
}

impl Register<0x01> {
    fn flags(&self) -> RegisterTwoFlags {
        RegisterTwoFlags::from_bits_retain(u16::from_be_bytes(self.data))
    }

    pub fn burst_sel(&self) -> BurstSel {
        let x = self.flags().contains(RegisterTwoFlags::BurstSelX);
        let y = self.flags().contains(RegisterTwoFlags::BurstSelY);
        let z = self.flags().contains(RegisterTwoFlags::BurstSelZ);
        let temp = self.flags().contains(RegisterTwoFlags::BurstSelT);
        BurstSel { x, y, z, temp }
    }

    pub fn temperature_compensation(&self) -> TemperatureCompensation {
        match self.flags().contains(RegisterTwoFlags::TcmpEn) {
            true => TemperatureCompensation::Enabled,
            false => TemperatureCompensation::Disabled,
        }
    }

    pub fn external_trigger(&self) -> bool {
        self.flags().contains(RegisterTwoFlags::ExtTrig)
    }

    pub fn wake_on_change_diff(&self) -> bool {
        self.flags().contains(RegisterTwoFlags::WOCDiff)
    }

    pub fn trigger_interrupt(&self) -> bool {
        self.flags().contains(RegisterTwoFlags::TrigInt)
    }
}

impl Register<0x02> {
    pub fn resolution(&self) -> Res3D {
        Res3D::from_u8_slice(&self.data)
    }
}

impl Register<0x03> {
    pub fn temperature_offset(&self) -> TempOffset {
        TempOffset::from_u8_slice(&self.data)
    }
}

impl Register<0x24> {
    pub fn temperature_reference(&self) -> TempRef {
        TempRef::from_u8_slice(&self.data)
    }
}

#[derive(Clone, Copy, Format)]
pub struct TempOffset {
    pub offset: [u8; 2],
}
impl TempOffset {
    pub fn from_u8_slice(offset: &[u8; 2]) -> Self {
        Self { offset: *offset }
    }
}

#[derive(Clone, Copy, Format)]
pub struct TempRef {
    pub offset: [u8; 2],
}
impl TempRef {
    pub fn from_u8_slice(offset: &[u8; 2]) -> Self {
        Self { offset: *offset }
    }
}

#[derive(Clone, Copy, Format)]
#[repr(usize)]
pub enum ZSeries {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, Format)]
#[repr(usize)]
pub enum Bist {
    Disabled,
    Enabled,
}
bitflags! {
    pub struct RegisterOneFlags: u16 {
        const ZSeries = 0b0000_0000_1000_0000;
        const Bist = 0b0000_0001_0000_0000;
    }
}

bitflags! {
    pub struct RegisterTwoFlags: u16 {
        const TrigInt = 0b1000_0000_0000_0000;
        const WOCDiff = 0b0001_0000_0000_0000;
        const ExtTrig = 0b0000_1000_0000_0000;
        const TcmpEn = 0b0000_0100_0000_0000;
        const BurstSelZ = 0b0000_0010_0000_0000;
        const BurstSelY = 0b0000_0001_0000_0000;
        const BurstSelX = 0b0000_0000_1000_0000;
        const BurstSelT = 0b0000_0000_0100_0000;
    }
}

#[derive(Clone, Copy, Format)]
#[repr(usize)]
pub enum TemperatureCompensation {
    Disabled,
    Enabled,
}

impl TemperatureCompensation {
    #[bitmatch]
    pub fn from_u8_slice(val: &[u8; 2]) -> Self {
        #[bitmatch]
        let "????_?t??" = val[1];
        #[bitmatch]
        match t {
            "1" => Self::Enabled,
            "0" => Self::Disabled,
        }
    }
}

#[derive(Clone, Copy, Format)]
#[repr(usize)]
pub enum Gain {
    ZERO,
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
}
impl Gain {
    #[bitmatch]
    pub fn from_u8_slice(val: &[u8; 2]) -> Self {
        #[bitmatch]
        match val[1] {
            "?000_????" => Self::ZERO,
            "?001_????" => Self::ONE,
            "?010_????" => Self::TWO,
            "?011_????" => Self::THREE,
            "?100_????" => Self::FOUR,
            "?101_????" => Self::FIVE,
            "?110_????" => Self::SIX,
            "?111_????" => Self::SEVEN,
        }
    }
}

#[derive(Clone, Copy, Format)]
#[repr(usize)]
pub enum Resolution {
    BIT19,
    BIT18,
    BIT17,
    BIT16,
}

#[derive(Clone, Copy, Format)]
#[repr(usize)]
pub enum HallConf {
    TWOPHASE,
    FOURPHASE,
}

impl HallConf {
    #[bitmatch]
    pub fn from_u8_slice(val: &[u8; 2]) -> Option<Self> {
        #[bitmatch]
        match val[1] {
            "????_0000" => Some(Self::TWOPHASE),
            "????_1100" => Some(Self::FOURPHASE),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Format)]
pub struct Res3D {
    pub x: Resolution,
    pub y: Resolution,
    pub z: Resolution,
}

impl Res3D {
    #[bitmatch]
    pub fn from_u8_slice(val: &[u8; 2]) -> Self {
        #[bitmatch]
        let "yxx?_????" = val[1];
        #[bitmatch]
        let "????_??zzv" = val[0];
        let xval = #[bitmatch]
        match x {
            "00" => Resolution::BIT19,
            "01" => Resolution::BIT18,
            "10" => Resolution::BIT17,
            "11" => Resolution::BIT16,
        };
        let yval = #[bitmatch]
        match v {
            "0" =>
            {
                #[bitmatch]
                match y {
                    "0" => Resolution::BIT19,
                    "1" => Resolution::BIT18,
                }
            }
            "1" =>
            {
                #[bitmatch]
                match y {
                    "0" => Resolution::BIT17,
                    "1" => Resolution::BIT16,
                }
            }
        };
        let zval = #[bitmatch]
        match z {
            "00" => Resolution::BIT19,
            "01" => Resolution::BIT18,
            "10" => Resolution::BIT17,
            "11" => Resolution::BIT16,
        };

        Self {
            x: xval,
            y: yval,
            z: zval,
        }
    }
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

struct MemoryLocation {
    register: u8,
    position: usize,
    length: usize,
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
