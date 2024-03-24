use crate::{hint, types, util, BitRead, BitWrite, Error, Parcel, Settings};

/// A newtype wrapping `Vec<T>` but with a custom length prefix type.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vec<S: types::Integer, T: Parcel> {
    /// The inner `Vec<T>`.
    pub elements: std::vec::Vec<T>,
    _a: std::marker::PhantomData<S>,
}

impl<S: types::Integer, T: Parcel> Vec<S, T> {
    /// Creates a new `Vec` from a list of elements.
    pub fn new(elements: std::vec::Vec<T>) -> Self {
        Vec {
            elements,
            _a: std::marker::PhantomData,
        }
    }
}

impl<S: types::Integer, T: Parcel> Parcel for Vec<S, T> {
    fn read_field(
        read: &mut dyn BitRead,
        settings: &Settings,
        hints: &mut hint::Hints,
    ) -> Result<Self, Error> {
        let elements = util::read_list_ext::<S, T>(read, settings, hints)?;
        Ok(Self::new(elements))
    }

    fn write_field(
        &self,
        write: &mut dyn BitWrite,
        settings: &Settings,
        hints: &mut hint::Hints,
    ) -> Result<(), Error> {
        util::write_list_ext::<S, T, _>(self.elements.iter(), write, settings, hints)
    }
}

impl<S, T> std::fmt::Debug for Vec<S, T>
where
    S: types::Integer,
    T: Parcel + std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.elements.fmt(fmt)
    }
}

impl<S, T> std::ops::Deref for Vec<S, T>
where
    S: types::Integer,
    T: Parcel,
{
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.elements[..]
    }
}

impl<S, T> std::ops::DerefMut for Vec<S, T>
where
    S: types::Integer,
    T: Parcel,
{
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.elements
    }
}

impl<S, T> AsRef<[T]> for Vec<S, T>
where
    S: types::Integer,
    T: Parcel,
{
    fn as_ref(&self) -> &[T] {
        &self.elements[..]
    }
}

impl<S, T> AsMut<[T]> for Vec<S, T>
where
    S: types::Integer,
    T: Parcel,
{
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.elements
    }
}

/// Stuff relating to `std::vec::Vec<T>`.
mod std_vec {
    use crate::{hint, util, BitRead, BitWrite, Error, FlexibleArrayMember, Parcel, Settings};

    impl<T: Parcel> Parcel for Vec<T> {
        fn read_field(
            read: &mut dyn BitRead,
            settings: &Settings,
            hints: &mut hint::Hints,
        ) -> Result<Self, Error> {
            util::read_list(read, settings, hints)
        }

        fn write_field(
            &self,
            write: &mut dyn BitWrite,
            settings: &Settings,
            hints: &mut hint::Hints,
        ) -> Result<(), Error> {
            util::write_list(self.iter(), write, settings, hints)
        }
    }

    impl<T: Parcel> FlexibleArrayMember for Vec<T> {}
}
