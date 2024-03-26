use bitstream_io::{BigEndian, BitWriter};
use core::any::Any;

use crate::{
    externally_length_prefixed::FieldLength, BitRead, BitWrite, Error, ExternallyLengthPrefixed,
    Protocol, Settings,
};
use std::{marker, mem};

/// A value that is aligned to a specified number of bytes.
///
/// When bytes are written, they are zero-padding at the end
/// until the total size is the smallest multiple of the
/// size of `ToSizeOfType`.
///
/// When an `Aligned` type is read, a value of the inner `T`
/// is first read, and then the minimum number of zero bytes in
/// order to maintain alignment are read and ignored.
///
/// Type parameters:
///
///   * `T` - The `Protocol` type that is to be transmitted
///   * `ToSizeOfType` The transmitted bytes will be aligned to a multiple
///     of `size_of::<ToSizeOfType>()`. For example, if `ToSizeOfType = u32`,
///     then the written bytes will be aligned to a multiple of 4 bytes.
///
/// Examples:
///
/// ```
/// # use bin_proto::Protocol;
/// #[derive(bin_proto::Protocol, Clone, Debug, PartialEq)]
/// struct Packet {
///     /// The length of the 'reason' string.
///     pub reason_length: u8,
///     /// The version number of the bin_proto.
///     pub version_number: (u32, u32),
///     #[protocol(length_prefix(bytes(reason_length)))]
///     pub reason: bin_proto::types::Aligned<String, u64>,
///
/// }
///
/// let bytes = Packet {
///     reason_length: 12,
///     version_number: (11, 0xdeadbeef),
///     reason: "hello world!".to_owned().into(),
/// }.bytes(&bin_proto::Settings::default()).unwrap();
///
/// assert_eq!(&[
///     12, // reason length
///     0, 0, 0, 11, 0xde, 0xad, 0xbe, 0xef, // version number
///     // the string "hello world".
///     b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', b'!',
///     0x00, 0x00, 0x00, 0x00, // padding bytes to align to string to 16 bytes.
///     ], &bytes[..]);
/// ```

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Aligned<T, ToSizeOfType>
where
    T: Protocol,
    ToSizeOfType: Sized,
{
    /// The inner value.
    pub value: T,
    _phantom: marker::PhantomData<ToSizeOfType>,
}

impl<T, ToSizeOfType> Aligned<T, ToSizeOfType>
where
    T: Protocol,
    ToSizeOfType: Sized,
{
    /// Creates a new aligned value.
    pub fn new(value: T) -> Self {
        Aligned {
            value,
            _phantom: marker::PhantomData,
        }
    }

    /// Gets the number of bytes of the alignment.
    pub fn alignment_bytes() -> usize {
        mem::size_of::<ToSizeOfType>()
    }
}

impl<T, ToSizeOfType> Protocol for Aligned<T, ToSizeOfType>
where
    T: Protocol,
    ToSizeOfType: Sized,
{
    fn read(read: &mut dyn BitRead, settings: &Settings, ctx: &mut dyn Any) -> Result<Self, Error> {
        let inner_value = T::read(read, settings, ctx)?;
        let value_size = inner_value.bytes_ctx(settings, ctx).unwrap().len();
        let padding_size = calculate_padding(Self::alignment_bytes(), value_size);

        for _ in 0..padding_size {
            let padding_byte = u8::read(read, settings, ctx)?;

            if padding_byte != 0x00 {
                return Err(Error::NonZeroPad);
            }
        }

        Ok(Aligned {
            value: inner_value,
            _phantom: marker::PhantomData,
        })
    }

    fn write(
        &self,
        write: &mut dyn BitWrite,
        settings: &Settings,
        ctx: &mut dyn Any,
    ) -> Result<(), Error> {
        let unaligned_bytes = self.value.bytes_ctx(settings, ctx)?;
        let aligned_bytes = align_to(Self::alignment_bytes(), 0x00, unaligned_bytes);
        write.write_bytes(&aligned_bytes)?;
        Ok(())
    }
}

impl<T, ToSizeOfType> ExternallyLengthPrefixed for Aligned<T, ToSizeOfType>
where
    T: Protocol + ExternallyLengthPrefixed,
    ToSizeOfType: Sized,
{
    fn read(
        read: &mut dyn BitRead,
        settings: &Settings,
        ctx: &mut dyn Any,
        length: &FieldLength,
    ) -> Result<Self, Error> {
        let inner_value = <T as ExternallyLengthPrefixed>::read(read, settings, ctx, length)?;
        let value_size = inner_value.bytes_ctx(settings, ctx).unwrap().len();
        let padding_size = calculate_padding(Self::alignment_bytes(), value_size);

        for _ in 0..padding_size {
            let padding_byte = u8::read(read, settings, ctx)?;

            if padding_byte != 0x00 {
                return Err(Error::NonZeroPad);
            }
        }

        Ok(Aligned {
            value: inner_value,
            _phantom: marker::PhantomData,
        })
    }

    fn write(
        &self,
        write: &mut dyn BitWrite,
        settings: &Settings,
        ctx: &mut dyn Any,
        length: &FieldLength,
    ) -> Result<(), Error> {
        let mut unaligned_bytes: Vec<u8> = Vec::new();
        ExternallyLengthPrefixed::write(
            &self.value,
            &mut BitWriter::endian(&mut unaligned_bytes, BigEndian),
            settings,
            ctx,
            length,
        )?;
        let aligned_bytes = align_to(Self::alignment_bytes(), 0x00, unaligned_bytes);
        write.write_bytes(&aligned_bytes)?;
        Ok(())
    }
}

impl<T, ToSizeOfType> From<T> for Aligned<T, ToSizeOfType>
where
    T: Protocol,
    ToSizeOfType: Sized,
{
    fn from(value: T) -> Self {
        Aligned {
            value,
            _phantom: marker::PhantomData,
        }
    }
}

/// Aligns a set of bytes to a multiple of the specified alignment.
fn align_to(align_to: usize, padding_byte: u8, bytes: Vec<u8>) -> Vec<u8> {
    // Thanks for the formula Ned!
    // https://stackoverflow.com/a/11642218
    let extra_padding_needed = calculate_padding(align_to, bytes.len());

    let extra_padding = (0..).take(extra_padding_needed).map(|_| padding_byte);

    let bytes: Vec<_> = bytes.into_iter().chain(extra_padding).collect();
    assert_eq!(0, bytes.len() % align_to, "failed to align");
    bytes
}

fn calculate_padding(align_to: usize, unaligned_size: usize) -> usize {
    // Thanks for the formula Ned!
    // https://stackoverflow.com/a/11642218
    (align_to - (unaligned_size % align_to)) % align_to
}

#[cfg(test)]
mod test {
    use super::*;

    mod alignment_calculations {
        use super::*;

        #[test]
        fn test_aligning_when_none_needed() {
            assert_eq!(vec![1, 2], align_to(1, 0x00, vec![1, 2]));
            assert_eq!(vec![1, 2], align_to(2, 0x00, vec![1, 2]));
        }

        #[test]
        fn test_align_to_3_with_size_2() {
            assert_eq!(vec![1, 2, 0], align_to(3, 0x00, vec![1, 2]));
        }

        #[test]
        fn test_align_to_4_with_size_2() {
            assert_eq!(vec![1, 2, 0xff, 0xff], align_to(4, 0xff, vec![1, 2]));
        }

        #[test]
        fn test_align_to_3_with_size_5() {
            assert_eq!(
                vec![1, 2, 3, 4, 5, 0],
                align_to(3, 0x00, vec![1, 2, 3, 4, 5])
            );
        }

        #[test]
        fn test_align_to_4_with_size_97() {
            let original = [1; 97];
            let aligned = align_to(4, 0x00, original.to_vec());

            let count_ones = aligned.iter().filter(|&&i| i == 1).count();
            let count_zeros = aligned.iter().filter(|&&i| i == 0).count();

            assert_eq!(97, count_ones);
            assert_eq!(3, count_zeros);
        }
    }
}
