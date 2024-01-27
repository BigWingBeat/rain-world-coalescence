//! A trait to check if two given types are the same type - primarily for use in generic contexts,
//! to have different behaviour if the generic type is a specific concrete type.
//!
//! This is necessary because the std library's `is` functions are only implemented for trait objects of `dyn Any`,
//! and not for concrete sized types

use std::any::TypeId;

pub trait Is {
    fn is<T: 'static>() -> bool;
}

impl<T: 'static> Is for T {
    #[inline(always)]
    fn is<U: 'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<U>()
    }
}
