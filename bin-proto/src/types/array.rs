use crate::{util, BitRead, BitWrite, Error, Protocol, Settings};

macro_rules! impl_parcel_for_array {
    ($n:expr) => {
        impl<T: Protocol> Protocol for [T; $n]
        where
            T: Copy,
        {
            fn read(read: &mut dyn BitRead, settings: &Settings) -> Result<Self, Error> {
                use std::mem;

                let elements: Vec<_> = util::read_items($n, read, settings)?.collect();
                assert_eq!(
                    elements.len(),
                    $n,
                    "fixed size array did not read the expected number of elements"
                );

                // N.B. We could potentially leave this array uninitialized
                // as an optimization.
                let mut uninit_array: [mem::MaybeUninit<T>; $n] =
                    unsafe { mem::MaybeUninit::uninit().assume_init() };
                for (i, element) in elements.into_iter().enumerate() {
                    uninit_array[i] = mem::MaybeUninit::new(element);
                }

                let array: &[T; $n] = unsafe { mem::transmute(&uninit_array) };
                Ok(*array)
            }

            fn write(
                &self,
                write: &mut dyn BitWrite,
                settings: &Settings,
            ) -> Result<(), Error> {
                util::write_items(self.iter(), write, settings)
            }
        }
    };
}

impl_parcel_for_array!(1);
impl_parcel_for_array!(2);
impl_parcel_for_array!(3);
impl_parcel_for_array!(4);
impl_parcel_for_array!(5);
impl_parcel_for_array!(6);
impl_parcel_for_array!(7);
impl_parcel_for_array!(8);
impl_parcel_for_array!(9);
impl_parcel_for_array!(10);
impl_parcel_for_array!(11);
impl_parcel_for_array!(12);
impl_parcel_for_array!(13);
impl_parcel_for_array!(14);
impl_parcel_for_array!(15);
impl_parcel_for_array!(16);
impl_parcel_for_array!(17);
impl_parcel_for_array!(18);
impl_parcel_for_array!(19);
impl_parcel_for_array!(20);
impl_parcel_for_array!(21);
impl_parcel_for_array!(22);
impl_parcel_for_array!(23);
impl_parcel_for_array!(24);
impl_parcel_for_array!(25);
impl_parcel_for_array!(26);
impl_parcel_for_array!(27);
impl_parcel_for_array!(28);
impl_parcel_for_array!(29);
impl_parcel_for_array!(30);
impl_parcel_for_array!(31);
impl_parcel_for_array!(32);
impl_parcel_for_array!(40);
impl_parcel_for_array!(42);
impl_parcel_for_array!(48);
impl_parcel_for_array!(64);
impl_parcel_for_array!(80);
impl_parcel_for_array!(120);
impl_parcel_for_array!(128);
impl_parcel_for_array!(256);
impl_parcel_for_array!(500);
impl_parcel_for_array!(512);
impl_parcel_for_array!(1000);
impl_parcel_for_array!(1024);
impl_parcel_for_array!(4096);
impl_parcel_for_array!(0xffff);

#[cfg(test)]
mod test {
    use bitstream_io::{BigEndian, BitReader, BitWriter};

    use crate::{Protocol, Settings};
    use std::io::Cursor;

    #[test]
    fn can_read_array() {
        let mut data = BitReader::endian(Cursor::new([0u8, 1, 2, 3]), BigEndian);
        let read_back: [u8; 4] = Protocol::read(&mut data, &Settings::default()).unwrap();
        assert_eq!(read_back, [0, 1, 2, 3]);
    }

    #[test]
    fn can_write_array() {
        let mut data = Vec::new();
        let mut writer = BitWriter::endian(&mut data, BigEndian);

        [5u8, 7, 9, 11]
            .write(&mut writer, &Settings::default())
            .unwrap();
        assert_eq!(data, vec![5, 7, 9, 11]);
    }
}
