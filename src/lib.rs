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
/// # fn main() {
/// let v = vec![1, 2, 3];
/// let index = 0;  // I.e., some computed value.
///
/// assume!(
///     unsafe: index < v.len(),
///     "index {} beyond v length",
///     index,
/// );
/// let element = v[index];  // Bounds check elided in release builds.
/// # }
/// ```
/// ```
/// # #[macro_use] extern crate assume;
/// # fn main() {
/// let mut v = vec![1, 2, 3];
/// let last_opt = v.pop();
///
/// assume!(
///     unsafe: last_opt.is_some(),
///     "vec missing element",
/// );
/// let last = last_opt.unwrap();  // Panic check elided in release builds.
/// # }
/// ```
/// ```
/// # #[macro_use] extern crate assume;
/// # fn main() {
/// let mut v = vec![1, 2, 3];
/// match v.pop() {
///     Some(value) => { /* ... */},
///     None => {
///         assume!(
///             unsafe: @unreachable,
///             "vec missing element"
///         );
///    }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! assume {
    (unsafe: $cond:expr $(,)?) => {{
        $crate::__impl_assume!($cond, "")
    }};
    (unsafe: $cond:expr, $fmt:expr $(, $($args:tt)*)?) => {{
        $crate::__impl_assume!($cond, concat!(": ", $fmt), $($($args)*)?)
    }};
    (unsafe: @unreachable $(,)?) => {{
        $crate::__impl_assume!(@unreachable, "")
    }};
    (unsafe: @unreachable, $fmt:expr $(, $($args:tt)*)?) => {{
        $crate::__impl_assume!(@unreachable, concat!(": ", $fmt), $($($args)*)?)
    }};
    (unsafe: $($_:tt)*) => {{
        compile_error!("assumption must be an expression or @unreachable");
    }};
    ($($_:tt)*) => {{
        compile_error!("assumption must be prefixed with 'unsafe: '");
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_assume {
    ($cond:expr, $fmt:expr $(, $($args:tt)*)?) => {{
        unsafe {
            if !$cond {
                if cfg!(debug_assertions) {
                    panic!(concat!(
                        "assumption failed: {}", $fmt), stringify!($cond), $($($args)*)?);
                } else {
                    core::hint::unreachable_unchecked()
                }
            }
        }
    }};
    (@unreachable, $fmt:expr $(, $($args:tt)*)?) => {{
        unsafe {
            if cfg!(debug_assertions) {
                panic!(concat!(
                    "assumption failed: @unreachable", $fmt), $($($args)*)?);
            } else {
                core::hint::unreachable_unchecked()
            }
        }
    }};
}
