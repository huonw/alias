//! `alias` offers some basic ways to mutate data while
//! aliased. [![crates.io](https://img.shields.io/crates/v/alias.svg)](https://crates.io/crates/alias)
//!
//! [*Source*](https://github.com/huonw/alias)
//!
//! # Examples
//!
//! ```rust
//! let mut x = 0;
//!
//! let y = alias::one(&mut x);
//! let z = y;
//!
//! // now we can read/write through multiple references
//! z.set(10);
//! y.set(y.get() + 2);
//! assert_eq!(z.get(), 12);
//! ```
//!
//! ```rust
//! let mut x = [0, 0, 0, 0];
//!
//! let y = alias::slice(&mut x);
//! let z = y;
//!
//! // now we can read/write through multiple references
//! for i in 0..4 {
//!     z[i].set(10);
//!     y[i].set(y[i].get() + i);
//! }
//!
//! assert_eq!(z[0].get(), 10);
//! assert_eq!(z[1].get(), 11);
//! assert_eq!(z[2].get(), 12);
//! assert_eq!(z[3].get(), 13);
//! ```
//!
//! # How is this OK?
//!
//! Rust's safety guarantees hinge around control how data is
//! aliased/can be manipulated while aliased. Key to this are the `&`
//! (shared/"immutable") and `&mut` (unique/mutable) reference types.
//!
//! The latter essentially has the guarantee that if `x: &mut T` is
//! accessible, then it is the only usable path to the `T` to which it
//! points. This ensures arbitrary mutation is entirely safe,
//! e.g. there's no way to invalidate other references because there
//! are no other references.
//!
//! On the other hand, `&T` references can be arbitrarily aliased
//! (possibly in a large number of threads), and so mutation cannot
//! occur by default. However, it can occur via specialised types that
//! control what mutation can happen, such as
//! `std::cell::Cell<T>`. That type is a plain wrapper around `T` that
//! only works with a subset of possible `T`s (`T: Copy`). These types
//! all assume they have full control over access to their internal
//! data: they mediate every interaction.
//!
//! If one has unique access to some piece of data (`&mut T`), it is
//! definitely safe to treat it as aliased (`&T`), but it is also safe
//! to treat it as aliased and mutable (`&Cell<T>`). No other piece of
//! code can be manipulating the `T` via any other path while the
//! `&mut T` reference exists (and lifetimes ensures `&Cell<T>` cannot
//! outlive it), so no other piece of code can do anything that
//! violates the assumption that the `Cell` controls every
//! interaction.
//!
//! This also relies on `T` → `Cell<T>` being a valid transmute, that
//! is, the layouts being identical. Strictly speaking, this isn't
//! guaranteed, but it is likely for it to remain this way. (There's
//! an additional factor of `Cell` theoretically having more layout
//! optimisations possible due to the way it restricts access to its
//! internals.)

use std::mem;
use std::cell::Cell;

/// Allow the mutable reference `data` to be mutated while aliased.
///
/// # Examples
///
/// ```rust
/// let mut x = 0;
///
/// let y = alias::one(&mut x);
/// let z = y;
///
/// // now we can read/write through multiple references
/// z.set(10);
/// y.set(y.get() + 2);
/// assert_eq!(z.get(), 12);
/// ```
pub fn one<'a, T: Copy>(data: &'a mut T) -> &'a Cell<T> {
    unsafe { mem::transmute(data) }
}

/// Allow the contents of the mutable slice `data` to be mutated while
/// aliased.
///
/// # Examples
///
/// ```rust
/// let mut x = [0, 0, 0, 0];
///
/// let y = alias::slice(&mut x);
/// let z = y;
///
/// // now we can read/write through multiple references
/// for i in 0..4 {
///     z[i].set(10);
///     y[i].set(y[i].get() + i);
/// }
///
/// assert_eq!(z[0].get(), 10);
/// assert_eq!(z[1].get(), 11);
/// assert_eq!(z[2].get(), 12);
/// assert_eq!(z[3].get(), 13);
/// ```
pub fn slice<'a, T: Copy>(data: &'a mut [T]) -> &'a [Cell<T>] {
    unsafe { mem::transmute(data) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    struct X<'a> {
        x: u8,
        y: u64,
        z: i8,
        w: &'a u32
    }

    #[test]
    fn smoke_one() {
        let a = 1;
        let b = 2;
        let val = X { x: 0xAA, y: !0, z: 0x77, w: &a };
        let val2 = X { x: 0x33, y: !0, z: 0x55, w: &b };
        let mut x = Some(val);

        {
            let y = one(&mut x);
            let z = y;
            y.set(z.get());
            assert_eq!(y.get(), Some(val));
            assert_eq!(z.get(), Some(val));

            z.set(Some(X { x: 1, .. y.get().unwrap()}));
            assert_eq!(y.get(), Some(X { x: 1, .. val }));
            assert_eq!(z.get(), Some(X { x: 1, .. val }));

            z.set(None);
            assert!(y.get().is_none());
            assert!(z.get().is_none());

            y.set(Some(val2));
            assert_eq!(y.get(), Some(val2));
            assert_eq!(z.get(), Some(val2));
        }

        assert_eq!(x, Some(val2));
    }
    #[test]
    fn smoke_slice() {
        let a = 1;
        let b = 2;
        let val = X { x: 0xAA, y: !0, z: 0x77, w: &a };
        let val2 = X { x: 0x33, y: !0, z: 0x55, w: &b };

        let mut x = [Some(val), Some(val2), None];

        {
            let y = slice(&mut x);
            let z = y;
            assert_eq!(y.len(), 3);

            let y0 = &y[0];
            let y1 = &y[1];
            let y2 = &y[2];
            let z0 = &z[0];
            let z1 = &z[1];
            let z2 = &z[2];

            y0.set(z0.get());
            assert_eq!(y0.get(), Some(val));
            assert_eq!(z0.get(), Some(val));

            y1.set(None);
            assert!(y1.get().is_none());
            assert!(z1.get().is_none());

            z2.set(Some(val2));
            assert_eq!(y2.get(), Some(val2));
            assert_eq!(z2.get(), Some(val2));
        }
        assert_eq!(x, [Some(val), None, Some(val2)]);
    }
}
