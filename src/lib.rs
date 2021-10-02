//! A macro for stating unsafe assumptions in Rust.
//!
//! Using this macro, one can supply assumptions to the compiler for use in optimization. These
//! assumptions are checked in `debug_assertion` configurations, and are unchecked (but still
//! present) otherwise.
//!
//! This is an inherently unsafe operation. It lives in the space between regular `assert!` and
//! pure `unsafe` accesses - it relies heavily on an optimizing compiler's ability to track
//! unreachable paths to eliminate unnecessary asserts.
//!
//! # Examples:
//! ```
//! # fn get_index() -> usize { 0 }
//! # fn main() {
//! use assume::assume;
//!
//! let v = vec![1, 2, 3];
//!
//! // Some computed index that, per invariants, is always in bounds.
//! let i = get_index();
//!
//! assume!(
//!     unsafe: i < v.len(),
//!     "index {} is beyond vec length",
//!     i,
//! );
//! let element = v[i];  // Bounds check optimized out per assumption.
//! # }
//! ```
//! ```
//! # use std::collections::HashMap;
//! # fn populate_items() -> HashMap<u32, String> {
//! #     let mut result = HashMap::default();
//! #     result.insert(0, "hello".to_string());
//! #     result
//! # }
//! # fn main() {
//! use assume::assume;
//!
//! let items: HashMap<u32, String> = populate_items();
//!
//! // Some item that, per invariants, always exists.
//! let item_zero_opt: Option<&String> = items.get(&0);
//!
//! assume!(
//!     unsafe: item_zero_opt.is_some(),
//!     "item zero missing from items map",
//! );
//! let item_zero = item_zero_opt.unwrap();  // Panic check optimized out per assumption.
//! # }
//! ```
//! ```
//! # fn main() {
//! use assume::assume;
//!
//! enum Choices {
//!     This,
//!     That,
//!     Other,
//! }
//! # fn get_choice() -> Choices { Choices::This }
//!
//! // Some choice that, per invariants, is never Other.
//! let choice = get_choice();
//!
//! match choice {
//!     Choices::This => { /* ... */ },
//!     Choices::That => { /* ... */ },
//!     Choices::Other => {
//!         // This case optimized out entirely, no panic emitted.
//!         assume!(
//!             unsafe: @unreachable,
//!             "choice was other",
//!         );
//!     },
//! }
//! # }
//! ```
//!
//! # Gotchas
//! - Unlike `debug_assert!` et. al., the condition of an `assume!` is always present.
//!   Complicated assumptions involving function calls and side effects are unlikely
//!   to be unhelpful in any case, but be aware they will run (unless the compiler can
//!   prove it is not needed). The assumed expression ought to be trivial and involve
//!   only the immediately available facts to guarantee this.
//!
//! - As stated, this relies on the optimizer to propagate the asumption. Differences
//!   in optimization level or mood of the compiler may cause it to fail to elide assertions
//!   in the final output. You are expected to benchmark and analyze the output yourself.
//!   If you simply *must* have no checking and do not want to rely on optimizations, then
//!   a `debug_assert!` + `unchecked` access is the way to go.
//!
//! - Avoid using `assume!(unsafe: false)` to indicate unreachable code. Although this works,
//!   the return type is `()` and not `!`, so the unreachability is not expressed to the compiler.
//!   This can result in warnings, or errors if e.g. different branches are computing some
//!    specific value. Use `assume!(unsafe: @unreachable)` instead.
//!
#![doc(html_root_url = "https://docs.rs/assume/0.3.0")]
#![no_std]

/// Assumes that the given condition is true.
///
/// This macro allows the expression of invariants in code. For example, one might `assume!` that
/// an index is in bounds prior to indexing into a slice - this would allow the optimizer to remove
/// the bounds checking entirely, under the promises of assume. In `debug_assertion` configurations
/// the expression is checked. Otherwise, it is unchecked.
///
/// Use `@unreachable` as the condition to assume the code path cannot be reached.
///
/// Because this expresses unchecked information, the act of assuming is inherently unsafe. The
/// safe (i.e., runtime checked) alternative to this is the [`assert!`] macro. If the condition
/// is `@unreachable`, the safe alternative to this is the [`unreachable!`] macro.
///
/// # Examples:
/// ```
/// # #[macro_use] extern crate assume;
/// # fn get_index() -> usize { 0 }
/// # fn main() {
/// let v = vec![1, 2, 3];
///
/// // Some computed index that, per invariants, is always in bounds.
/// let i = get_index();
///
/// assume!(
///     unsafe: i < v.len(),
///     "index {} is beyond vec length",
///     i,
/// );
/// let element = v[i];  // Bounds check optimized out per assumption.
/// # }
/// ```
/// ```
/// # #[macro_use] extern crate assume;
/// # use std::collections::HashMap;
/// # fn populate_items() -> HashMap<u32, String> {
/// #     let mut result = HashMap::default();
/// #     result.insert(0, "hello".to_string());
/// #     result
/// # }
/// # fn main() {
/// let items: HashMap<u32, String> = populate_items();
///
/// // Some item that, per invariants, always exists.
/// let item_zero_opt: Option<&String> = items.get(&0);
///
/// assume!(
///     unsafe: item_zero_opt.is_some(),
///     "item zero missing from items map",
/// );
/// let item_zero = item_zero_opt.unwrap();  // Panic check optimized out per assumption.
/// # }
/// ```
/// ```
/// # #[macro_use] extern crate assume;
/// # fn main() {
/// enum Choices {
///     This,
///     That,
///     Other,
/// }
/// # fn get_choice() -> Choices { Choices::This }
///
/// // Some choice that, per invariants, is never Other.
/// let choice = get_choice();
///
/// match choice {
///     Choices::This => { /* ... */ },
///     Choices::That => { /* ... */ },
///     Choices::Other => {
///         // This case optimized out entirely, no panic emitted.
///         assume!(
///             unsafe: @unreachable,
///             "choice was other",
///         );
///     },
/// }
/// # }
/// ```
#[macro_export]
macro_rules! assume {
    (unsafe: $cond:expr $(,)?) => {{
        $crate::__assume_impl!($cond, "", "")
    }};
    (unsafe: $cond:expr, $fmt:expr $(, $($args:tt)*)?) => {{
        $crate::__assume_impl!($cond, ": ", $fmt, $($($args)*)?)
    }};
    (unsafe: @unreachable $(,)?) => {{
        $crate::__assume_impl!(@unreachable, "unreachable", "", "")
    }};
    (unsafe: @unreachable, $fmt:expr $(, $($args:tt)*)?) => {{
        $crate::__assume_impl!(@unreachable, "unreachable", ": ", $fmt, $($($args)*)?)
    }};
    (unsafe: $($_:tt)*) => {{
        $crate::__private::compile_error!("assumption must be an expression or @unreachable");
    }};
    ($($_:tt)*) => {{
        $crate::__private::compile_error!("assumption must be prefixed with 'unsafe: '");
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __assume_impl {
    ($cond:expr, $fmt:expr $(, $($args:tt)*)?) => {{
        #[allow(unused_unsafe)]
        if unsafe { !$cond } {
            $crate::__assume_impl!(
                @unreachable, $crate::__private::stringify!($cond), $fmt, $($($args)*)?)
        }
    }};
    (@unreachable, $what:expr, $sep:expr, $fmt:expr $(, $($args:tt)*)?) => {{
        #[cfg(debug_assertions)]
        {
            // We could put $what and $sep into concat!, as they are strings,
            // but this generates erroneous rust analyzer errors:
            //     https://github.com/rust-analyzer/rust-analyzer/issues/10300
            $crate::__private::panic!($crate::__private::concat!(
                "assumption failed: {}{}", $fmt),
                $what,
                $sep,
                $($($args)*)?
            );
        }

        #[cfg(not(debug_assertions))]
        unsafe {
            $crate::__private::unreachable_unchecked()
        }
    }};
}

/// Used by macros.
#[doc(hidden)]
pub mod __private {
    pub use core::{compile_error, concat, hint::unreachable_unchecked, panic, stringify};
}

#[cfg(test)]
mod tests {
    /// Rogue macro.
    #[allow(unused_macros)]
    macro_rules! panic {
        ($($tt:tt)*) => {
            return
        };
    }

    /// Rogue macro.
    #[allow(unused_macros)]
    macro_rules! concat {
        ($($tt:tt)*) => {
            return
        };
    }

    /// Rogue macro.
    #[allow(unused_macros)]
    macro_rules! stringify {
        ($($tt:tt)*) => {
            return
        };
    }

    /// Rogue macro.
    #[allow(unused_macros)]
    macro_rules! cfg {
        ($($tt:tt)*) => {
            return
        };
    }

    mod core {}

    #[test]
    fn conditional_can_be_unsafe() {
        let values = [1, 2, 3];
        assume!(unsafe: *values.get_unchecked(0) > 0);
    }

    #[test]
    #[should_panic(expected = "assumption failed: 2 > 3")]
    #[cfg(debug_assertions)]
    fn is_not_affected_by_call_site_environment() {
        assume!(unsafe: 2 > 3);
    }

    #[test]
    #[should_panic(expected = "assumption failed: 2 > 3: oh no")]
    #[cfg(debug_assertions)]
    fn is_not_affected_by_call_site_environment_with_message() {
        assume!(unsafe: 2 > 3, "oh no");
    }

    #[test]
    #[should_panic(expected = "assumption failed: 2 > 3: oh no, a problem")]
    #[cfg(debug_assertions)]
    fn is_not_affected_by_call_site_environment_with_format() {
        assume!(unsafe: 2 > 3, "oh no, a {}", "problem");
    }

    #[test]
    #[should_panic(expected = "assumption failed: unreachable")]
    #[cfg(debug_assertions)]
    fn is_not_affected_by_call_site_environment_unreachable() {
        assume!(unsafe: @unreachable);
    }

    #[test]
    #[should_panic(expected = "assumption failed: unreachable: oh no")]
    #[cfg(debug_assertions)]
    fn is_not_affected_by_call_site_environment_unreachable_with_message() {
        assume!(unsafe: @unreachable, "oh no");
    }

    #[test]
    #[should_panic(expected = "assumption failed: unreachable: oh no, a problem")]
    #[cfg(debug_assertions)]
    fn is_not_affected_by_call_site_environment_unreachable_with_format() {
        assume!(unsafe: @unreachable, "oh no, a {}", "problem");
    }
}
