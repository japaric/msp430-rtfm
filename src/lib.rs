#![feature(const_fn)]
#![feature(proc_macro)]
#![no_std]

extern crate msp430;
extern crate msp430_rtfm_macros;
extern crate static_ref;

use core::cell::UnsafeCell;

#[doc(hidden)]
pub use msp430::interrupt::CriticalSection;
#[doc(hidden)]
pub use msp430::interrupt::enable;
pub use msp430::interrupt::free as atomic;
pub use msp430_rtfm_macros::app;
#[doc(hidden)]
pub use static_ref::Static;

pub struct Resource<T> {
    data: UnsafeCell<T>,
}

impl<T> Resource<T> {
    pub const fn new(value: T) -> Self {
        Resource {
            data: UnsafeCell::new(value),
        }
    }

    pub fn borrow<'cs>(
        &'static self,
        _cs: &'cs CriticalSection,
    ) -> &'cs Static<T> {
        unsafe { Static::ref_(&*self.data.get()) }
    }

    pub unsafe fn borrow_mut<'cs>(
        &'static self,
        _cs: &'cs CriticalSection,
    ) -> &'cs mut Static<T> {
        Static::ref_mut(&mut *self.data.get())
    }

    pub fn get(&self) -> *mut T {
        self.data.get()
    }
}

unsafe impl<T> Sync for Resource<T> {}

#[macro_export]
macro_rules! task {
    ($NAME:ident, $body:path) => {
        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "msp430-interrupt" fn $NAME() {
            let f: fn(::$NAME::Resources) = $body;

            f(
                ::$NAME::Resources::new(),
            );
        }
    };
    ($NAME:ident, $body:path, $local:ident {
        $($var:ident: $ty:ty = $expr:expr;)+
    }) => {
        struct $local {
            $($var: $ty,)+
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "msp430-interrupt" fn $NAME() {
            let f: fn(
                &mut $local,
                ::$NAME::Resources,
            ) = $body;

            static mut LOCAL: $local = $local {
                $($var: $expr,)+
            };

            f(
                &mut LOCAL,
                ::$NAME::Resources::new(),
            );
        }
    };

}
