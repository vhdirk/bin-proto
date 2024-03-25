use crate::{hint, BitField, BitRead, BitWrite, Error, Parcel, Settings};

impl<T: Parcel> Parcel for Option<T> {
    fn read_field(
        read: &mut dyn BitRead,
        settings: &Settings,
        _: &mut hint::Hints,
    ) -> Result<Self, Error> {
        let is_some = bool::read(read, settings)?;

        if is_some {
            let value = T::read(read, settings)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn write_field(
        &self,
        write: &mut dyn BitWrite,
        settings: &Settings,
        _: &mut hint::Hints,
    ) -> Result<(), Error> {
        self.is_some().write(write, settings)?;

        if let Some(ref value) = *self {
            value.write(write, settings)?;
        }
        Ok(())
    }
}

impl<T: Parcel> BitField for Option<T> {
    fn read_field(
        read: &mut dyn BitRead,
        settings: &Settings,
        hints: &mut hint::Hints,
        bits: u32,
    ) -> Result<Self, Error> {
        let is_some = <bool as BitField>::read_field(read, settings, hints, bits)?;

        if is_some {
            let value = T::read(read, settings)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn write_field(
        &self,
        write: &mut dyn BitWrite,
        settings: &Settings,
        hints: &mut hint::Hints,
        bits: u32,
    ) -> Result<(), Error> {
        BitField::write_field(&self.is_some(), write, settings, hints, bits)?;

        if let Some(ref value) = *self {
            value.write(write, settings)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use bitstream_io::{BigEndian, BitReader, BitWriter};

    use crate::hint::Hints;

    use super::*;

    #[test]
    fn can_read_some() {
        assert_eq!(
            Option::<u8>::from_raw_bytes(&[1, 5], &Settings::default()).unwrap(),
            Some(5)
        )
    }

    #[test]
    fn can_read_none() {
        assert_eq!(
            Option::<u8>::from_raw_bytes(&[0], &Settings::default()).unwrap(),
            None
        )
    }

    #[test]
    fn can_write_some() {
        assert_eq!(Some(5u8).raw_bytes(&Settings::default()).unwrap(), &[1, 5])
    }

    #[test]
    fn can_write_none() {
        assert_eq!(None::<u8>.raw_bytes(&Settings::default()).unwrap(), &[0])
    }

    #[test]
    fn can_read_some_bitfield() {
        assert_eq!(
            <Option::<u8> as BitField>::read_field(
                &mut BitReader::endian(Cursor::new([0x82u8, 0x80]), BigEndian),
                &Settings::default(),
                &mut Hints::default(),
                1,
            )
            .unwrap(),
            Some(5)
        )
    }

    #[test]
    fn can_read_none_bitfield() {
        assert_eq!(
            <Option::<u8> as BitField>::read_field(
                &mut BitReader::endian(Cursor::new([0x00]), BigEndian),
                &Settings::default(),
                &mut Hints::default(),
                1,
            )
            .unwrap(),
            None
        )
    }

    #[test]
    fn can_write_some_bitfield() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = BitWriter::endian(&mut buffer, BigEndian);
        BitField::write_field(
            &Some(5u8),
            &mut writer,
            &Settings::default(),
            &mut Hints::default(),
            1,
        )
        .unwrap();
        writer.byte_align().unwrap();
        assert_eq!(vec![0x82, 0x80], buffer)
    }

    #[test]
    fn can_write_none_bitfield() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = BitWriter::endian(&mut buffer, BigEndian);
        BitField::write_field(
            &None::<u8>,
            &mut writer,
            &Settings::default(),
            &mut Hints::default(),
            1,
        )
        .unwrap();
        writer.byte_align().unwrap();
        assert_eq!(vec![0x00], buffer)
    }
}
