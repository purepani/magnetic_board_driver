use bitflags::{Flags, bitflags};
use bitmatch::bitmatch;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub trait MaskingBits
where
    Self: Sized + TryFromBytes + IntoBytes + Immutable
{
    const MASK: u16;

    fn transmute(val: &u16) -> Self {
        const { assert!(core::mem::size_of::<u16>() == core::mem::size_of::<Self>()) };
        let val = val & Self::MASK;
        let bytes = val.as_bytes();
        Self::try_read_from_bytes(bytes).expect("Mask is incorrect or invalid value found.")
    }

    fn to_u16(&self) -> u16 {
        const { assert!(core::mem::size_of::<u16>() == core::mem::size_of::<Self>()) };
        let mut val: u16 = 0;
        self.write_to(val.as_mut_bytes()).expect("Wrong size for MaskingBits::u16.");
        val
    }
}




#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, TryFromBytes, IntoBytes)]
#[repr(u16)]
pub enum ZSeries {
    Disabled = 0b00000000,
    Enabled = 0b10000000,
}

impl MaskingBits for ZSeries {
    const MASK: u16 = 0b10000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, TryFromBytes, IntoBytes)]
#[repr(u16)]
pub enum Gain {
    ZERO = 0 << 4,
    ONE = 1 << 4,
    TWO = 2 << 4,
    THREE = 3 << 4,
    FOUR = 4 << 4,
    FIVE = 5 << 4,
    SIX = 6 << 4,
    SEVEN = 7 << 4,
}

impl MaskingBits for Gain {
    const MASK: u16 = 0b01110000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, TryFromBytes, IntoBytes)]
#[repr(u16)]
pub enum HallConf {
    TWOPHASE = 0b0000,
    FOURPHASE = 0b1100,
}

impl MaskingBits for HallConf {
    const MASK: u16 = 0b1111;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, TryFromBytes, IntoBytes)]
#[repr(u16)]
pub enum Bist {
    Disabled = 0b0_00000000,
    Enabled = 0b1_00000000,
}

impl MaskingBits for Bist {
    const MASK: u16 = 0b1_00000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(transparent)]
pub struct BurstDataRate(u16);
impl MaskingBits for BurstDataRate {
    const MASK: u16 = 0b111111;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes)]
#[repr(u16)]
enum BurstSelFlags {
    T = 1 << 6,
    X = 1 << 7,
    Y = 1 << 8,
    Z = 1 << 9,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(transparent)]
pub struct BurstSel {
    flags: u16,
}

impl MaskingBits for BurstSel {
    const MASK: u16 = 0b00000011_11000000;
}

impl BurstSel {
    pub fn t(&self) -> bool {
        const MASK: u16 = BurstSelFlags::T as u16;
        self.flags & MASK == MASK
    }
    pub fn x(&self) -> bool {
        const MASK: u16 = BurstSelFlags::X as u16;
        self.flags & MASK == MASK
    }
    pub fn y(&self) -> bool {
        const MASK: u16 = BurstSelFlags::Y as u16;
        self.flags & MASK == MASK
    }
    pub fn z(&self) -> bool {
        const MASK: u16 = BurstSelFlags::Z as u16;
        self.flags & MASK == MASK
    }
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum TemperatureCompensation {
    Disabled = 0 << 10,
    Enabled = 1 << 10,
}

impl MaskingBits for TemperatureCompensation {
    const MASK: u16 = 0b00000100_00000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum ExternalTrigger {
    Disabled = 0 << 11,
    Enabled = 1 << 11,
}

impl MaskingBits for ExternalTrigger {
    const MASK: u16 = 0b00001000_00000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum WakeOnChangeDiffMode {
    Disabled = 0 << 12,
    Enabled = 1 << 12,
}

impl MaskingBits for WakeOnChangeDiffMode {
    const MASK: u16 = 0b00010000_00000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum CommMode {
    SPI = 0b10 << 13,
    I2C = 0b11 << 13,
    BOTH1 = (0b00 << 13),
    BOTH2 = (0b01 << 13),
}
impl MaskingBits for CommMode {
    const MASK: u16 = 0b01100000_00000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum TriggerInterrupt {
    Disabled = 0 << 15,
    Enabled = 1 << 15,
}

impl MaskingBits for TriggerInterrupt {
    const MASK: u16 = 0b10000000_00000000;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(transparent)]
pub struct OSR(u16);
impl MaskingBits for OSR {
    const MASK: u16 = 0b00000000_00000011;
}

impl OSR {
    pub fn over_sampling_ratio(&self) -> u16 {
        self.0
    }
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(transparent)]
pub struct OSR2(u16);
impl MaskingBits for OSR2 {
    const MASK: u16 = 0b00011000_00000000;
}

impl OSR2 {
    pub fn over_sampling_ratio(&self) -> u16 {
        self.0
    }
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum DigFilt {
    Filt0 = 0b000 << 2,
    Filt1 = 0b001 << 2,
    Filt2 = 0b010 << 2,
    Filt3 = 0b011 << 2,
    Filt4 = 0b100 << 2,
    Filt5 = 0b101 << 2,
    Filt6 = 0b110 << 2,
    Filt7 = 0b111 << 2,
}

impl MaskingBits for DigFilt {
    const MASK: u16 = 0b00000000_00011100;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(u16)]
pub enum Resolution {
    BIT19,
    BIT18,
    BIT17,
    BIT16,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(usize)]
enum Res3DFlagShift {
    X = 5,
    Y = 7,
    Z = 9,
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(transparent)]
pub struct Res3D(u16);

impl MaskingBits for Res3D {
    const MASK: u16 = 0b00000111_11100000;
}

impl Res3D {
    pub fn x(&self) -> Resolution {
        let res = (self.0 >> Res3DFlagShift::X as usize) & 0b11;
        zerocopy::try_transmute!(res).expect("Couldn't transmute resolution X.")
    }
    pub fn y(&self) -> Resolution {
        let res = (self.0 >> Res3DFlagShift::Y as usize) & 0b11;
        zerocopy::try_transmute!(res).expect("Couldn't transmute resolution Y.")
    }
    pub fn z(&self) -> Resolution {
        let res = (self.0 >> Res3DFlagShift::Z as usize) & 0b11;
        zerocopy::try_transmute!(res).expect("Couldn't transmute resolution Z.")
    }
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct SensitivityTemperatureCompensation {
    high_threshold: u8,
    low_threshold: u8,
}
impl MaskingBits for SensitivityTemperatureCompensation {
    const MASK: u16 = u16::MAX;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct OffsetX(u16);
impl MaskingBits for OffsetX {
    const MASK: u16 = u16::MAX;
}
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct OffsetY(u16);
impl MaskingBits for OffsetY {
    const MASK: u16 = u16::MAX;
}
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct OffsetZ(u16);
impl MaskingBits for OffsetZ {
    const MASK: u16 = u16::MAX;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct WakeOnXYThreshold(u16);
impl MaskingBits for WakeOnXYThreshold {
    const MASK: u16 = u16::MAX;
}
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct WakeOnZThreshold(u16);
impl MaskingBits for WakeOnZThreshold {
    const MASK: u16 = u16::MAX;
}
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct WakeOnTempThreshold(u16);
impl MaskingBits for WakeOnTempThreshold {
    const MASK: u16 = u16::MAX;
}

#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable, IntoBytes, TryFromBytes)]
#[repr(C)]
pub struct TemperatureRef(u16);
impl MaskingBits for TemperatureRef {
    const MASK: u16 = u16::MAX;
}


#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
#[derive(Clone, Copy, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct Register<const R: usize>(u16);

impl<const R: usize> Register<R> {
    pub fn get<RF>(&self) -> RF
    where
        RF: MaskingBits + RegisterField<R>,
    {
        RF::transmute(&self.0)
    }

    pub fn set<RF>(&self, val: RF) -> Self
    where
        RF: MaskingBits + RegisterField<R>,
    {
        Self((self.0 & !RF::MASK) | val.to_u16())
    }

}

pub trait RegisterField<const R: usize> {}
impl RegisterField<0> for ZSeries {}
impl RegisterField<0> for Bist {}
impl RegisterField<0> for HallConf {}
impl RegisterField<0> for Gain {}

impl RegisterField<1> for BurstDataRate {}
impl RegisterField<1> for BurstSel {}
impl RegisterField<1> for TemperatureCompensation {}
impl RegisterField<1> for ExternalTrigger {}
impl RegisterField<1> for WakeOnChangeDiffMode {}
impl RegisterField<1> for CommMode {}
impl RegisterField<1> for TriggerInterrupt {}

impl RegisterField<2> for OSR {}
impl RegisterField<2> for OSR2 {}
impl RegisterField<2> for DigFilt {}
impl RegisterField<2> for Res3D {}

impl RegisterField<3> for SensitivityTemperatureCompensation {}
impl RegisterField<4> for OffsetX {}
impl RegisterField<5> for OffsetY {}
impl RegisterField<6> for OffsetZ {}
impl RegisterField<7> for WakeOnXYThreshold {}
impl RegisterField<8> for WakeOnZThreshold {}
impl RegisterField<9> for WakeOnTempThreshold {}
impl RegisterField<24> for TemperatureRef {}

impl Register<0> {
    pub fn gain(&self) -> Gain {
        self.get()
    }

    pub fn hall_conf(&self) -> HallConf {
        self.get()
    }

    pub fn z_series(&self) -> ZSeries {
        self.get()
    }

    pub fn bist(&self) -> Bist {
        self.get()
    }
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
