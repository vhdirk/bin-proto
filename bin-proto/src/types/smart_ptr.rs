use crate::{BitRead, BitWrite, Error, Protocol, Settings};

use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

macro_rules! impl_smart_ptr_type {
    ($ty:ident) => {
        impl<T: Protocol> Protocol for $ty<T> {
            fn read(read: &mut dyn BitRead, settings: &Settings) -> Result<Self, Error> {
                let value = T::read(read, settings)?;
                Ok($ty::new(value))
            }

            fn write(
                &self,
                write: &mut dyn BitWrite,
                settings: &Settings,
            ) -> Result<(), Error> {
                self.deref().write(write, settings)
            }
        }
    };
}

impl_smart_ptr_type!(Rc);
impl_smart_ptr_type!(Arc);
