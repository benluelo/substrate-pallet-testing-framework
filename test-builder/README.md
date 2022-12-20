# test-builder

This crate provides a build-script utility to aggregate functions marked with
the `#[subtest::test]` annotation into a macro_rules! that can then be
instantiated with any runtime.

_NOTE: The `#[subtest::test]` proc-macro annotations are not yet implemented,
currently the build script looks for public functions in public modules with one
generic parameter (not great, but it works) so make sure that helper functions
are private and at the top level!_

## Overview

There are several problems when attempting to create runtime-agnostic tests:

- normal `#[test]` functions can't be generic, even with
  `custom_test_frameworks`
- anything under `#[cfg(test)]` is only available _within_ the crate

The solution is to write testing functions _not_ under `#[cfg(test)]`, but
rather under a _feature_ (`#[cfg(feature = "testing")]`), which are themselves
generic over the runtime.

These next issue is actually calling these functions in a `#[test]`. Take the
following example:

```rust
// some-pallet/src/testing.rs:

pub fn test_a<T>() {
    // snip
}

pub fn test_b<T>() {
    // snip
}
```

```rust
// some-runtime/src/tests.rs

#[test]
fn test_a<T>() {
    some_pallet::testing::test_a<crate::Runtime>();
}

#[test]
fn test_b<T>() {
    some_pallet::testing::test_b<crate::Runtime>();
}
```

This is a lot of very repetative boilerplate, and requires adding new tests to
every runtime using these tests, even potential downstream runtimes!

To mitigate this, a build script is used that reads all of the exposed pallet
tests and exposes them through a `macro_rules!`. With the previously shown
functions, this time with the `#[subtest::test]` macro annotation:

```rust
// testing.rs

#[subtest::test]
pub fn test_a<T>() {
    // snip
}

#[subtest::test]
pub fn test_b<T>() {
    // snip
}
```

These functions are then aggregated into a macro_rules! that emulates the
original module structure of the tests, which are then called with the runtime
"filled in" with the runtime provided:

```rust
macro_rules! tests {
    (mod $mod_name<$Runtime>) => {
        mod $mod_name {
            #[test]
            fn test_a() {
                $crate::testing::test_a<$Runtime>()
            }

            #[test]
            fn test_b() {
                $crate::testing::test_b<$Runtime>()
            }
        }
    }
}
```
