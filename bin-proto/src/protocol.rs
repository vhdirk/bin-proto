use bitstream_io::{BigEndian, BitReader, BitWriter, LittleEndian};

use crate::{BitRead, BitWrite, Error, Settings};
use std::io;

/// A trait for bit-level co/dec.
pub trait Protocol: Sized {
    /// Reads self from a stream.
    ///
    /// Blocks until a value is received.
    fn read(read: &mut dyn BitRead, settings: &Settings) -> Result<Self, Error>;

    /// Writes a value to a stream.
    fn write(&self, write: &mut dyn BitWrite, settings: &Settings) -> Result<(), Error>;

    /// Parses a new value from its raw byte representation.
    fn from_bytes(bytes: &[u8], settings: &Settings) -> Result<Self, Error> {
        match settings.byte_order {
            crate::ByteOrder::LittleEndian => {
                let mut buffer = BitReader::endian(io::Cursor::new(bytes), LittleEndian);
                Self::read(&mut buffer, settings)
            }
            crate::ByteOrder::BigEndian => {
                let mut buffer = BitReader::endian(io::Cursor::new(bytes), BigEndian);
                Self::read(&mut buffer, settings)
            }
        }
    }

    /// Gets the raw bytes of this type as a field of a larger type.
    fn bytes(&self, settings: &Settings) -> Result<Vec<u8>, Error> {
        let mut data = Vec::new();
        match settings.byte_order {
            crate::ByteOrder::LittleEndian => {
                let mut writer = BitWriter::endian(&mut data, LittleEndian);
                self.write(&mut writer, settings)?;
                writer.byte_align()?;
            }
            crate::ByteOrder::BigEndian => {
                let mut writer = BitWriter::endian(&mut data, BigEndian);
                self.write(&mut writer, settings)?;
                writer.byte_align()?;
            }
        };

        Ok(data)
    }
}
