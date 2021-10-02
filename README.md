assume
======

A macro for stating unsafe assumptions in Rust.

Using this macro, one can supply assumptions to the compiler for use in optimization. These assumptions are checked in `debug_assertion` configurations, and are unchecked (but still present) otherwise.

This is an inherently unsafe operation. It lives in the space between regular `assert!` and pure `unsafe` accesses - it relies heavily on an optimizing compiler's ability to track unreachable paths to eliminate unnecessary asserts.

```toml
[dependencies]
assume = "0.3"
```

## Examples

```rust
use assume::assume;

let v = vec![1, 2, 3];

// Some computed index that, per invariants, is always in bounds.
let i = get_index();

assume!(
    unsafe: i < v.len(),
    "index {} is beyond vec length",
    i,
);
let element = v[i];  // Bounds check optimized out per assumption.
```

```rust
use assume::assume;

let items: HashMap<u32, String> = populate_items();

// Some item that, per invariants, always exists.
let item_zero_opt: Option<&String> = items.get(&0);

assume!(
    unsafe: item_zero_opt.is_some(),
    "item zero missing from items map",
);
let item_zero = item_zero_opt.unwrap();  // Panic check optimized out per assumption.
```

```rust
use assume::assume;

enum Choices {
    This,
    That,
    Other,
}

// Some choice that, per invariants, is never Other.
let choice = get_choice();

match choice {
    Choices::This => { /* ... */ },
    Choices::That => { /* ... */ },
    Choices::Other => {
        // This case optimized out entirely, no panic emitted.
        assume!(
            unsafe: @unreachable,
            "choice was other",
        );
    },
}
```

## Motivation

Often, programs have invariants that are not or cannot be expressed in the type system. Rust is safe by default, and asserts are made as needed to ensure this.

Consider the following (somewhat convoluted) example:

```rust
pub struct ValuesWithEvens {
    values: Vec<u32>,
    evens: Vec<usize>,
}

impl ValuesWithEvens {
    pub fn new(values: Vec<u32>) -> Self {
        let evens = values
            .iter()
            .enumerate()
            .filter_map(|(index, value)| if value % 2 == 0 { Some(index) } else { None })
            .collect();

        Self { values, evens }
    }

    pub fn pop_even(&mut self) -> Option<u32> {
        if let Some(index) = self.evens.pop() {
            // We know this index is valid, but a bounds check is performed anyway.
            let value = self.values[index];

            Some(value)
        } else {
            None
        }
    }
}

fn main() {
    let mut vwe = ValuesWithEvens::new(vec![1, 2, 3, 4]);

    let last_even = vwe.pop_even().unwrap();
    println!("{}", last_even);
}
```

By construction, indices contained within `evens` are always valid indices into `values`. However, as written there is a bounds check on the line:

```rust
let value = self.values[index];
```

This ensures a bug in the program does not result in an out of bounds access. (For example, if another method were introduced that modified `values` it could invalidate the indices - this would not result in undefined behavior thanks to bounds checking.)

However, if this is a hot-spot in the program we may want to remove this check. Sometimes this trade-off is necessary to achieve performance requirements. Rust offers `unsafe` access:

```rust
    pub fn pop_even(&mut self) -> Option<u32> {
        if let Some(index) = self.evens.pop() {
            let value = unsafe { *self.values.get_unchecked(index) };

            Some(value)
        } else {
            None
        }
    }
```

As expected this has no bounds check, but other than the `unsafe` keyword we've removed any scrutiny around the access. We can improve this by including a debug-only assertion that the index really is okay:

```rust
    pub fn pop_even(&mut self) -> Option<u32> {
        if let Some(index) = self.evens.pop() {
            debug_assert!(index < self.evens.len());
            let value = unsafe { *self.values.get_unchecked(index) };

            Some(value)
        } else {
            None
        }
    }
```

Can you spot the bug? We've asserted against the wrong vector! This should be:

```rust
debug_assert!(index < self.values.len());
//                         ^^^^^^
```

The decoupling of assertion to optimization is unwieldy and error-prone.

The `assume!` macro relies on the optimizer's ability to validate and use stated assumptions - an incorrect assumption will have no effect and the bounds check will remain in the program.

Using the `assume!` macro looks like:

```rust
    pub fn pop_even(&mut self) -> Option<u32> {
        if let Some(index) = self.evens.pop() {
            assume!(
                unsafe: index < self.evens.len(),
                "even index {} beyond values vec",
                index
            );
            let value = self.values[index];

            Some(value)
        } else {
            None
        }
    }
```

Now the optimizer is aware of what we believe to be true, and is checking that this expression implies the optimization we want. In this case it does, so the bounds check is removed. Furthermore, this will assert our condition holds in `debug_assertion` configurations (such as in tests).

Best of all, the code we actually want to write remains untouched and easy to read.

## When not to use

Do not use this macro.

Rely on `assert!` to check program invariants.

Rely on `unreachable!` to state that some code path should never be taken.

## When to use

Okay - once you:

- Have a reliable method for measuring your performance.
- Have profiling results indicating some invariant check is causing overhead.
- Have no way of re-arranging the program to express this without overhead.
- Are about to reach for an `unsafe` get operation.

Then you should consider `assume!` instead. This is more terse, leaves your safe code untouched, asserts in debug builds, and ensures runtime checks are removed only if they are implied by the assumption.

This is not a beginner-friendly macro; you are expected to be able to view disassembly and verify the desired optimizations are taking place. You are also expected to have a suite of tests that build with `debug_assertion` enabled in order to catch violations of the invariant.

## Gotchas

- Unlike `debug_assert!` et. al., the condition of an `assume!` is always present. Complicated assumptions involving function calls and side effects are unlikely to be unhelpful in any case, but be aware they will run (unless the compiler can prove it is not needed). The assumed expression ought to be trivial and involve only the immediately available facts to guarantee this.

- As stated, this relies on the optimizer to propagate the asumption. Differences in optimization level or mood of the compiler may cause it to fail to elide assertions in the final output. You are expected to benchmark and analyze the output yourself. If you simply *must* have no checking and do not want to rely on optimizations, then a `debug_assert!` + `unchecked` access is the way to go.

- Avoid using `assume!(unsafe: false)` to indicate unreachable code. Although this works, the return type is `()` and not `!`, so the unreachability is not expressed to the compiler. This can result in warnings, or errors if e.g. different branches are computing some specific value. Use `assume!(unsafe: @unreachable)` instead.

## See Also

The underlying mechanism for the macro is [`std::hint::unreachable_unchecked`](https://doc.rust-lang.org/std/hint/fn.unreachable_unchecked.html).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.