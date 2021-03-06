// use std::marker::PhantomData;
use libc::c_uint;
use std::fmt;
use std::slice;

use crate::errors::LiquidError;
use crate::liquid_dsp_sys as raw;
use crate::utils::{ToCPointer, ToCPointerMut, ToCValue};
use num::complex::Complex32;

pub struct CbufferRf {
    inner: raw::cbufferf,
    num_elements: u32,
}

pub struct CbufferCf {
    inner: raw::cbuffercf,
    num_elements: u32,
}

macro_rules! cbuffer_xxx_impl {
    ($obj:ty, (
        $create:expr, $create_max:expr,
        $reset:expr, $size:expr,
        $max_size:expr,$max_read:expr,
        $space_available:expr,$is_full:expr,
        $debug_print:expr,$release:expr,
        $push:expr, $write:expr,
        $pop:expr, $read:expr,
        $destroy:expr, $type:ty)) => {
        impl $obj {
            /// creates a circular buffer object that can hold up to *max_size* samples
            pub fn create(max_size: u32) -> Self {
                Self {
                    inner: unsafe { $create(max_size as _) },
                    num_elements: 0,
                }
            }

            /// create circular buffer object of a particular size
            ///
            /// and specify the maximum number of elements that can be read
            /// at any given time.
            pub fn create_max(max_size: u32, max_read: u32) -> Self {
                Self {
                    inner: unsafe { $create_max(max_size as _, max_read as _) },
                    num_elements: 0,
                }
            }

            pub fn reset(&mut self) {
                unsafe {
                    $reset(self.inner);
                }
            }

            ///  returns the number of elements currently in the buffer
            pub fn size(&self) -> u32 {
                unsafe { $size(self.inner) as u32 }
            }

            /// returns the maximum number of elements the buffer can hold
            pub fn max_size(&self) -> u32 {
                unsafe { $max_size(self.inner) as u32 }
            }

            /// Returns the maximum number of elements that can be read from
            /// the buffer at any given time.
            pub fn max_read(&self) -> u32 {
                unsafe { $max_read(self.inner) as u32 }
            }

            /// return number of elements available for writing
            pub fn space_available(&self) -> u32 {
                unsafe { $space_available(self.inner) as u32 }
            }

            // TODO check it
            pub fn is_full(&self) -> bool {
                unsafe { $is_full(self.inner) == 1 }
            }

            /// print cbuffer object properties and internal state
            pub fn debug_print(&self) {
                unsafe {
                    $debug_print(self.inner);
                }
            }

            pub fn release(&mut self, n: usize) -> Result<(), LiquidError> {
                if n > self.num_elements as usize {
                    return Err(LiquidError::EmptyBuffer);
                }
                unsafe {
                    $release(self.inner, n as c_uint);
                    Ok(())
                }
            }
            /// write a single sample into the buffer
            pub fn push(&mut self, v: $type) -> Result<(), &'static str> {
                if self.num_elements == self.max_size() {
                    return Err("warning: no space available");
                }
                unsafe {
                    $push(self.inner, v.to_c_value());
                    self.num_elements += 1;
                    Ok(())
                }
            }

            // write samples from the buffer
            pub fn write(&mut self, buffer: &[$type]) -> Result<(), &'static str> {
                if buffer.len() > self.space_available() as usize {
                    return Err("cannot write more elements than are available");
                }
                unsafe {
                    $write(
                        self.inner,
                        buffer.to_ptr() as *mut _,
                        buffer.len() as c_uint,
                    );
                    self.num_elements += buffer.len() as u32;
                    Ok(())
                }
            }

            /// remove and return a single element from the buffer
            pub fn pop(&mut self) -> Option<$type> {
                if self.num_elements == 0u32 {
                    return None;
                }
                unsafe {
                    let mut out = <$type>::default();
                    $pop(self.inner, out.to_ptr_mut());
                    self.num_elements -= 1;
                    Some(out)
                }
            }

            pub fn read(&self) -> &[$type] {
                let ptr = &mut <$type>::default().to_ptr_mut() as *mut _;
                let mut len = 0u32;
                unsafe {
                    $read(
                        self.inner,
                        self.num_elements as c_uint,
                        ptr,
                        &mut len as *mut _,
                    );
                    slice::from_raw_parts(*ptr as *const _, len as usize)
                }
            }
        }

        impl fmt::Debug for $obj {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "cbuffer: {} [max size: {}, max read: {}, elements: {}]:\n",
                    stringify!($obj),
                    self.max_size(),
                    self.max_read(),
                    self.num_elements
                )
            }
        }

        impl Drop for $obj {
            fn drop(&mut self) {
                unsafe {
                    $destroy(self.inner);
                }
            }
        }

        impl AsRef<[$type]> for $obj {
            #[inline]
            fn as_ref(&self) -> &[$type] {
                self.read()
            }
        }
    };
}


cbuffer_xxx_impl!(
    CbufferRf,
    (
        raw::cbufferf_create,
        raw::cbufferf_create_max,
        raw::cbufferf_reset,
        raw::cbufferf_size,
        raw::cbufferf_max_size,
        raw::cbufferf_max_read,
        raw::cbufferf_space_available,
        raw::cbufferf_is_full,
        raw::cbufferf_debug_print,
        raw::cbufferf_release,
        raw::cbufferf_push,
        raw::cbufferf_write,
        raw::cbufferf_pop,
        raw::cbufferf_read,
        raw::cbufferf_destroy,
        f32
    )
);

cbuffer_xxx_impl!(
    CbufferCf,
    (
        raw::cbuffercf_create,
        raw::cbuffercf_create_max,
        raw::cbuffercf_reset,
        raw::cbuffercf_size,
        raw::cbuffercf_max_size,
        raw::cbuffercf_max_read,
        raw::cbuffercf_space_available,
        raw::cbuffercf_is_full,
        raw::cbuffercf_debug_print,
        raw::cbuffercf_release,
        raw::cbuffercf_push,
        raw::cbuffercf_write,
        raw::cbuffercf_pop,
        raw::cbuffercf_read,
        raw::cbuffercf_destroy,
        Complex32
    )
);

#[cfg(test)]
mod tests {
    use super::CbufferRf;

    #[test]
    fn test_cbufferf() {
        let mut v = [1.2, 2.5, 3.6, 4.4, 5.8, 6.9, 7.8, 8.98];

        let mut cb = CbufferRf::create(10);

        cb.write(&mut v).unwrap();
        assert_eq!(cb.read(), &v);

        // release 2 elements from the buffer
        cb.release(2).unwrap();
        assert_eq!(cb.space_available(), 4);
    }
}
