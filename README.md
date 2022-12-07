# Substrate Pallet Testing Framework

A pallet testing framework, built around the philosophy of explicitness and
correctness.

## The Problem

Substrate unit tests are inherently difficult to write. The most common way to
do it is to create a mock runtime in the same directory as the pallet, and then
write tests specific to that runtime. There are several issues with this:

1. Maintaining runtimes, even mock ones, has a very low ROI. They are a lot of
   work to write and configure for very little payoff.
2. Mock runtimes aren't DRY - they typically contain code copied directly from
   the "actual" runtime(s).
3. Mock runtimes fall out of sync with other runtimes very easily, which can
   cause bugs if the runtime configuration is updated but the pallet's mock
   runtime is not.

## The Solution

Pallet unit tests currently test too much. They attempt to test algorithms,
runtime configurations, extrinsics, events emissions, general workflows, among
other things. I propose thhe following testing structure, inspired by/ based off
of how
[rust suggests tests be written](https://doc.rust-lang.org/rust-by-example/testing.html):

### Pallet Unit Tests

Unit tests should be reserved for testing _small units of functionalilty_, of
which should be completely separated from the pallet. These units will mostly be
algorithms used by the pallet. There should be no tests requiring a runtime
here. This is a very limited scope by design, as most pallet tests will be
written in the integration tests.

### Pallet Integration Tests

#### The Problem

Rust tests are compiled as a standalone binary per test; this means that they can't
be generic (and DI is impossible on the type level).

#### Possible solutions:

> Define test functions without the `#[test]` attribute, and manually call them in a test

Pros:

- Tests can be generic over the runtime, allowing for DI
- Tests aren't limited to being `fn() -> ()` (i.e. they can take args and return values)

Cons:

- Lots of manual work required to run tests. Consider the following example:

  ```rust
  // in some-pallet/src/testing.rs:
  fn test_something<T: Config>() {
     // snip
  }
  
  // in some-runtime/tests/some-pallet.rs:
  
  use crate::*;
  use some_pallet::testing::*;
  
  fn test_something() {
     test_something::<Runtime>();
  }
  ```
  
  The entire pallet's test suite has to be copy-pasted for every runtime it's tested on, which
  is very error prone and lots of work.
- Backtraces become slightly harder to read (although they're already not great with the externalities)
  TODO: Maybe write a backtrace parser? Something like https://github.com/auxoncorp/tnfilt

> Use a build.rs file to automatically generate something that can take a runtime type and create
a test suite

More information on build scripts: https://doc.rust-lang.org/cargo/reference/build-script-examples.html#code-generation

The build script could read the source code of the crate and generate a macro_rules! that would
generate the the boilerplate seen above.

Pros:

 - Simple end-user experience, just invoke a macro to generate a test suite for a pallet
 
Cons:

 - This requires the pallet developer to write the build script; however this could be abstracted
  away into something similar to https://github.com/paritytech/substrate/blob/11c50578549969979121577cde987ad3f9d95bd8/utils/wasm-builder/src/lib.rs

> Write a custom test harness

https://doc.rust-lang.org/unstable-book/language-features/custom-test-frameworks.html
https://rust-lang.github.io/rfcs/2318-custom-test-frameworks.html
https://os.phil-opp.com/testing/#custom-test-frameworks
https://www.infinyon.com/blog/2021/04/rust-custom-test-harness/

This sounds like fun

TODO: Finish writing this

## Current Features

- Storage changes assertions with the `Diffable` trait and
  `AssertableDiffableStorageAction`

## Roadmap

- Companion framework to make writing runtime-agnostic tests easier, utilizing
  the change-set interfaces to facilitate exhaustive assertions (as described in
  the solution section)

## Current Non-goals

These are current non-goals for this project. They may be supported in the
future.

- Full e2e testing with multiple nodes
