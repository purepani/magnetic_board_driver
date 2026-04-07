use bitflags::{bitflags, Flags};
use bitmatch::bitmatch;
use zerocopy::{ KnownLayout, Immutable, Unaligned};





#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable)]
#[repr(u16)]
pub enum ZSeries {
    Disabled=0b00000000,
    Enabled=0b10000000,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable)]
#[repr(u16)]
pub enum Gain {
    ZERO=0 << 4,
    ONE=1 << 4,
    TWO=2 << 4,
    THREE=3 << 4,
    FOUR=4 << 4,
    FIVE=5 << 4,
    SIX=6 << 4,
    SEVEN=7 << 4,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable)]
#[repr(u16)]
pub enum HallConf {
    TWOPHASE=0b0000,
    FOURPHASE=0b1100,
}
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable)]
#[repr(u16)]
pub enum Bist {
    Disabled=0b0_00000000,
    Enabled=0b1_00000000,
}







pub struct BurstSel {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub temp: bool,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct TempOffset {
    pub offset: [u8; 2],
}


#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct TempRef {
    pub offset: [u8; 2],
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
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
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, )]
#[repr(usize)]
pub enum Resolution {
    BIT19,
    BIT18,
    BIT17,
    BIT16,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct Res3D {
    pub x: Resolution,
    pub y: Resolution,
    pub z: Resolution,
}
