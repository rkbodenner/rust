// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!

The `ToBytes` and `IterBytes` traits

*/

use cast;
use io;
use io::Writer;
use option::{None, Option, Some};
use old_iter::BaseIter;
use str::StrSlice;

pub type Cb<'self> = &'self fn(buf: &[u8]) -> bool;

/**
 * A trait to implement in order to make a type hashable;
 * This works in combination with the trait `Hash::Hash`, and
 * may in the future be merged with that trait or otherwise
 * modified when default methods and trait inheritence are
 * completed.
 */
pub trait IterBytes {
    /**
     * Call the provided callback `f` one or more times with
     * byte-slices that should be used when computing a hash
     * value or otherwise "flattening" the structure into
     * a sequence of bytes. The `lsb0` parameter conveys
     * whether the caller is asking for little-endian bytes
     * (`true`) or big-endian (`false`); this should only be
     * relevant in implementations that represent a single
     * multi-byte datum such as a 32 bit integer or 64 bit
     * floating-point value. It can be safely ignored for
     * larger structured types as they are usually processed
     * left-to-right in declaration order, regardless of
     * underlying memory endianness.
     */
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool;
}

impl IterBytes for bool {
    #[inline]
    fn iter_bytes(&self, _lsb0: bool, f: Cb) -> bool {
        f([
            *self as u8
        ])
    }
}

impl IterBytes for u8 {
    #[inline]
    fn iter_bytes(&self, _lsb0: bool, f: Cb) -> bool {
        f([
            *self
        ])
    }
}

impl IterBytes for u16 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        if lsb0 {
            f([
                *self as u8,
                (*self >> 8) as u8
            ])
        } else {
            f([
                (*self >> 8) as u8,
                *self as u8
            ])
        }
    }
}

impl IterBytes for u32 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        if lsb0 {
            f([
                *self as u8,
                (*self >> 8) as u8,
                (*self >> 16) as u8,
                (*self >> 24) as u8,
            ])
        } else {
            f([
                (*self >> 24) as u8,
                (*self >> 16) as u8,
                (*self >> 8) as u8,
                *self as u8
            ])
        }
    }
}

impl IterBytes for u64 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        if lsb0 {
            f([
                *self as u8,
                (*self >> 8) as u8,
                (*self >> 16) as u8,
                (*self >> 24) as u8,
                (*self >> 32) as u8,
                (*self >> 40) as u8,
                (*self >> 48) as u8,
                (*self >> 56) as u8
            ])
        } else {
            f([
                (*self >> 56) as u8,
                (*self >> 48) as u8,
                (*self >> 40) as u8,
                (*self >> 32) as u8,
                (*self >> 24) as u8,
                (*self >> 16) as u8,
                (*self >> 8) as u8,
                *self as u8
            ])
        }
    }
}

impl IterBytes for i8 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u8).iter_bytes(lsb0, f)
    }
}

impl IterBytes for i16 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u16).iter_bytes(lsb0, f)
    }
}

impl IterBytes for i32 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u32).iter_bytes(lsb0, f)
    }
}

impl IterBytes for i64 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u64).iter_bytes(lsb0, f)
    }
}

impl IterBytes for char {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u32).iter_bytes(lsb0, f)
    }
}

#[cfg(target_word_size = "32")]
impl IterBytes for uint {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u32).iter_bytes(lsb0, f)
    }
}

#[cfg(target_word_size = "64")]
impl IterBytes for uint {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as u64).iter_bytes(lsb0, f)
    }
}

impl IterBytes for int {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as uint).iter_bytes(lsb0, f)
    }
}

impl IterBytes for float {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as f64).iter_bytes(lsb0, f)
    }
}

impl IterBytes for f32 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        let i: u32 = unsafe {
            // 0.0 == -0.0 so they should also have the same hashcode
            cast::transmute(if *self == -0.0 { 0.0 } else { *self })
        };
        i.iter_bytes(lsb0, f)
    }
}

impl IterBytes for f64 {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        let i: u64 = unsafe {
            // 0.0 == -0.0 so they should also have the same hashcode
            cast::transmute(if *self == -0.0 { 0.0 } else { *self })
        };
        i.iter_bytes(lsb0, f)
    }
}

impl<'self,A:IterBytes> IterBytes for &'self [A] {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        self.each(|elt| elt.iter_bytes(lsb0, |b| f(b)))
    }
}

impl<A:IterBytes,B:IterBytes> IterBytes for (A,B) {
  #[inline]
  fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
    match *self {
      (ref a, ref b) => { a.iter_bytes(lsb0, f) && b.iter_bytes(lsb0, f) }
    }
  }
}

impl<A:IterBytes,B:IterBytes,C:IterBytes> IterBytes for (A,B,C) {
  #[inline]
  fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
    match *self {
      (ref a, ref b, ref c) => {
        a.iter_bytes(lsb0, f) && b.iter_bytes(lsb0, f) && c.iter_bytes(lsb0, f)
      }
    }
  }
}

// Move this to vec, probably.
fn borrow<'x,A>(a: &'x [A]) -> &'x [A] {
    a
}

impl<A:IterBytes> IterBytes for ~[A] {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        borrow(*self).iter_bytes(lsb0, f)
    }
}

impl<A:IterBytes> IterBytes for @[A] {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        borrow(*self).iter_bytes(lsb0, f)
    }
}

impl<'self> IterBytes for &'self str {
    #[inline]
    fn iter_bytes(&self, _lsb0: bool, f: Cb) -> bool {
        f(self.as_bytes())
    }
}

impl IterBytes for ~str {
    #[inline]
    fn iter_bytes(&self, _lsb0: bool, f: Cb) -> bool {
        // this should possibly include the null terminator, but that
        // breaks .find_equiv on hashmaps.
        f(self.as_bytes())
    }
}

impl IterBytes for @str {
    #[inline]
    fn iter_bytes(&self, _lsb0: bool, f: Cb) -> bool {
        // this should possibly include the null terminator, but that
        // breaks .find_equiv on hashmaps.
        f(self.as_bytes())
    }
}

impl<A:IterBytes> IterBytes for Option<A> {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        match *self {
          Some(ref a) => 0u8.iter_bytes(lsb0, f) && a.iter_bytes(lsb0, f),
          None => 1u8.iter_bytes(lsb0, f)
        }
    }
}

impl<'self,A:IterBytes> IterBytes for &'self A {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (**self).iter_bytes(lsb0, f)
    }
}

impl<A:IterBytes> IterBytes for @A {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (**self).iter_bytes(lsb0, f)
    }
}

impl<A:IterBytes> IterBytes for ~A {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (**self).iter_bytes(lsb0, f)
    }
}

// NB: raw-pointer IterBytes does _not_ dereference
// to the target; it just gives you the pointer-bytes.
impl<A> IterBytes for *const A {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (*self as uint).iter_bytes(lsb0, f)
    }
}

/// A trait for converting a value to a list of bytes.
pub trait ToBytes {
    /// Converts the current value to a list of bytes. This is equivalent to
    /// invoking iter_bytes on a type and collecting all yielded values in an
    /// array
    fn to_bytes(&self, lsb0: bool) -> ~[u8];
}

impl<A:IterBytes> ToBytes for A {
    fn to_bytes(&self, lsb0: bool) -> ~[u8] {
        do io::with_bytes_writer |wr| {
            for self.iter_bytes(lsb0) |bytes| {
                wr.write(bytes)
            }
        }
    }
}
