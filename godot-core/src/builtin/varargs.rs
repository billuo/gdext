/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::Deref;

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
#[derive(Clone, Default)]
pub struct Varargs {
    args: Vec<Variant>,
}

impl Varargs {
    /// Number of collected arguments.
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Whether no extra arguments were passed.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Returns the argument at the given index, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&Variant> {
        self.args.get(index)
    }

    /// Iterates over the collected arguments as [`Variant`] references.
    pub fn iter(&self) -> std::slice::Iter<'_, Variant> {
        self.args.iter()
    }

    /// Converts the collected arguments into an untyped [`VarArray`] (allocating a new Godot array).
    pub fn to_var_array(&self) -> VarArray {
        VarArray::from(&self.args[..])
    }

    /// Collects `len` arguments starting at `args_ptr` into an owned `Varargs`.
    ///
    /// # Safety
    /// `args_ptr` must point to an array of at least `len` valid `GDExtensionConstVariantPtr` entries, each pointing to a live `Variant`
    /// for the duration of the call.
    pub(crate) unsafe fn from_var_arg_slice(
        args_ptr: *const sys::GDExtensionConstVariantPtr,
        len: usize,
    ) -> Self {
        let args = if len == 0 {
            Vec::new()
        } else {
            (0..len)
                .map(|i| {
                    // SAFETY: caller guarantees `args_ptr` holds `len` valid variant pointers.
                    let variant = unsafe { Variant::borrow_var_sys(*args_ptr.add(i)) };
                    variant.clone()
                })
                .collect()
        };

        Self { args }
    }
}

impl Deref for Varargs {
    type Target = [Variant];

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl AsRef<[Variant]> for Varargs {
    fn as_ref(&self) -> &[Variant] {
        &self.args
    }
}

impl std::fmt::Debug for Varargs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Varargs").field(&self.args).finish()
    }
}

impl<'a> IntoIterator for &'a Varargs {
    type Item = &'a Variant;
    type IntoIter = std::slice::Iter<'a, Variant>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

impl IntoIterator for Varargs {
    type Item = Variant;
    type IntoIter = std::vec::IntoIter<Variant>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}
