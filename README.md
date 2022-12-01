# Substrate Pallet Testing Framework

A pallet testing framework, built around the philosophy of explicitness and correctness.

## The Problem

Substrate unit tests are inherently difficult to write. The most common way to do it is to create a mock runtime in the same directory as the pallet, and then write tests specific to that runtime. There are several issues with this:

1. Maintaining runtimes, even mock ones, has a very low ROI. They are a lot of work to write and configure for very little payoff.
2. Mock runtimes aren't DRY - they typically contain code copied directly from the "actual" runtime(s).
3. Mock runtimes fall out of sync with other runtimes very easily, which can cause bugs if the runtime configuration is updated but the pallet's mock runtime is not.

## The Solution

Pallet unit tests currently test too much. They attempt to test algorithms, runtime configurations, extrinsics, events emissions, general workflows, among other things. I propose thhe following testing structure, inspired by/ based off of how [rust suggests tests be written](https://doc.rust-lang.org/rust-by-example/testing.html):

### Pallet Unit Tests

Unit tests should be reserved for testing *small units of functionalilty*, of which should be completely separated from the pallet. These units will mostly be algorithms used by the pallet. There should be no tests requiring a runtime here. This is a very limited scope by design, as most pallet tests will be written in the integration tests.

# Pallet Integration Tests

TODO: Finish writing this

## Current Features

- Storage changes assertions with the `Diffable` trait and `AssertableDiffableStorageAction`

## Roadmap

- Companion framework to make writing runtime-agnostic tests easier, utilizing the change-set interfaces to facilitate exhaustive assertions (as described in the solution section)

## Current Non-goals

These are current non-goals for this project. They may be supported in the future.

- Full e2e testing with multiple nodes
