/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use godot_ffi as sys;

use crate::builtin::{VarArray, Variant};

/// Variable-length argument list received by a `#[func]` method, making the method a GDScript-facing vararg function.
///
/// A `Varargs` parameter can only appear as the **last** parameter of a `#[func]` method. When Godot calls such a method,
/// any arguments beyond the declared (typed) parameters are collected into this container:
///
/// ```no_run
/// use godot::prelude::*;
/// # #[derive(GodotClass)]
/// # #[class(init)]
/// # struct MyClass;
/// #
/// # #[godot_api]
/// # impl MyClass {
/// #[func]
/// fn sum_all(&self, offset: i64, args: Varargs) -> i64 {
///     offset + args.iter().map(|v| v.to::<i64>()).sum::<i64>()
/// }
/// # }
/// ```
///
/// From GDScript, such a method can be called with any number of trailing arguments:
/// ```gdscript
/// obj.sum_all(10, 1, 2, 3) # returns 16
/// ```
///
/// `Varargs` **borrows** the arguments passed by Godot for the duration of the call; it does not allocate or copy them.
/// The only allocating operation is [`to_var_array()`][Self::to_var_array], which explicitly materializes a Godot array.
#[derive(Copy, Clone)]
pub struct Varargs<'a> {
    // Borrows the array of variant pointers handed to the varcall FFI callback. Each element points to a `Variant` owned by Godot,
    // valid for the duration of the call (lifetime `'a`).
    args: &'a [sys::GDExtensionConstVariantPtr],
}

impl<'a> Varargs<'a> {
    /// Number of collected arguments.
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Whether no extra arguments were passed.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Returns the argument at the given index, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&'a Variant> {
        self.args.get(index).map(|ptr| {
            // SAFETY: each pointer in `args` points to a live `Variant` for the lifetime `'a`, as guaranteed by `from_raw_parts()`.
            unsafe { Variant::borrow_var_sys(*ptr) }
        })
    }

    /// Iterates over the collected arguments as [`Variant`] references.
    pub fn iter(&self) -> VarargsIter<'a> {
        VarargsIter {
            inner: self.args.iter(),
        }
    }

    /// Converts the collected arguments into an untyped [`VarArray`], allocating a new Godot array.
    ///
    /// This is the only operation that allocates; iteration and indexing borrow the original arguments.
    pub fn to_var_array(&self) -> VarArray {
        self.iter().cloned().collect()
    }

    /// Constructs a borrowed `Varargs` from a raw pointer/length pair, without copying the arguments.
    ///
    /// # Safety
    /// `ptr` must point to an array of at least `len` valid `GDExtensionConstVariantPtr` entries, each pointing to a live `Variant`,
    /// valid for the lifetime `'a` chosen by the caller.
    #[doc(hidden)]
    pub unsafe fn from_raw_parts(ptr: *const sys::GDExtensionConstVariantPtr, len: usize) -> Self {
        // SAFETY: guaranteed by the caller.
        let args = unsafe { std::slice::from_raw_parts(ptr, len) };
        Self { args }
    }
}

impl<'a> IntoIterator for &Varargs<'a> {
    type Item = &'a Variant;
    type IntoIter = VarargsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::fmt::Debug for Varargs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// Iterator over the arguments of a [`Varargs`], yielding each as a borrowed [`Variant`].
pub struct VarargsIter<'a> {
    inner: std::slice::Iter<'a, sys::GDExtensionConstVariantPtr>,
}

impl<'a> Iterator for VarargsIter<'a> {
    type Item = &'a Variant;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|ptr| {
            // SAFETY: each pointer points to a live `Variant` for the lifetime `'a` (see `Varargs::from_raw_parts`).
            unsafe { Variant::borrow_var_sys(*ptr) }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for VarargsIter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}
