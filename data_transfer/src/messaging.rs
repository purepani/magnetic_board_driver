use defmt::write;
use serde::{Deserialize, Serialize};

use crate::conversions::MagneticField;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "use-std")]
extern crate std;

#[derive(Debug, defmt::Format)]
pub enum Error {
    FailedCRCSerialization,
    FailedCOBSSerialization,
    FailedCRCDeserialization,
    FailedCOBSDeserialization,
    FailedWrite,
    FailedRead,
    FailedParse(PostcardError),
}
#[derive(Debug)]
pub struct PostcardError(postcard::Error);

impl defmt::Format for PostcardError {
    fn format(&self, fmt: defmt::Formatter) {
        let Self(err) = self;
        match err {
            postcard::Error::WontImplement => write!(fmt, "Won't implmenet"),
            postcard::Error::NotYetImplemented => write!(fmt, "Not yet implmented"),
            postcard::Error::SerializeBufferFull => write!(fmt, "Serialize buffer full"),
            postcard::Error::SerializeSeqLengthUnknown => {
                write!(fmt, "Serialize Seq length unknown")
            }
            postcard::Error::DeserializeUnexpectedEnd => write!(fmt, "Deserialize unexpected end"),
            postcard::Error::DeserializeBadVarint => write!(fmt, "Deserialize bad varint"),
            postcard::Error::DeserializeBadBool => write!(fmt, "Deserialize bad bool"),
            postcard::Error::DeserializeBadChar => write!(fmt, "Deserialize bad char"),
            postcard::Error::DeserializeBadUtf8 => write!(fmt, "Deserialize bad utf8"),
            postcard::Error::DeserializeBadOption => write!(fmt, "Deserialize bad option"),
            postcard::Error::DeserializeBadEnum => write!(fmt, "Deserialize bad enum"),
            postcard::Error::DeserializeBadEncoding => write!(fmt, "Deserialize bad encoding"),
            postcard::Error::DeserializeBadCrc => write!(fmt, "Deserialize bad crc"),
            postcard::Error::SerdeSerCustom => write!(fmt, "Serde ser custom"),
            postcard::Error::SerdeDeCustom => write!(fmt, "Serde deser custom"),
            postcard::Error::CollectStrError => write!(fmt, "Collect str error"),
            _ => write!(fmt, "Unknown error"),
        }
    }
}

impl From<postcard::Error> for Error {
    fn from(value: postcard::Error) -> Self {
        Self::FailedParse(PostcardError(value))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub field: MagneticField,
    pub position: (f32, f32, f32),
}

impl Message {
    pub fn new(field: MagneticField, position: (f32, f32, f32)) -> Self {
        Self { field, position }
    }
    pub fn write_to<T: embedded_io::Write>(&self, writer: &mut T) -> Result<(), Error> {
        let mut cobs_buffer = [0; 64];
        //let c = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);
        //let crc_serialized = postcard::to_slice_crc32(self, &mut crc_buffer, digest)
        //   .map_err(|_| Error::FailedCRCSerialization)?;
        let cobs_serialized = postcard::to_slice_cobs(self, &mut cobs_buffer)
            .map_err(|_| Error::FailedCOBSSerialization)?;
        //let _ = postcard::to_eio(cobs_serialized, writer).map_err(|_| Error::FailedWrite)?;
        writer
            .write_all(cobs_serialized)
            .map_err(|_| Error::FailedWrite)?;
        Ok(())
    }

    #[cfg(feature = "use-std")]
    pub fn read<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let mut buffer = [0; 128];
        let _ = reader.read_exact(&mut buffer);
        let mut slice = buffer.split_mut(|val| *val == 0x00);
        let _ = slice.next();
        let msg = slice.next().ok_or(Error::FailedRead)?;

        let cobs = postcard::take_from_bytes_cobs::<Message>(msg)?;
        Ok(cobs.0)
    }
}
