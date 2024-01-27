//! Because the std library's `is` functions are only implemented for trait objects of `dyn Any`

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
