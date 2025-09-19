# Arkad

Arkad is a state machine library specifically designed for processing SEC filings in Rust. It provides a modular and extensible framework for building robust, type-safe and testable workflows for handling SEC data.

## Overview

Arkad consists of two Rust crates:

### State Maschine

`state_maschine` is a library for the general implementation of hierarchical finite state machines, meant to be extended for more specific use-cases.

### SEC

`sec` is a library that extends `state_maschine` for the specific purpose of processing SEC filings, from the acquisiton of the data, transforming the data into a suitable format for saving and saving the data itself onto storage. It is designed to be flexible and extendable.

## Usage

Arkad is still in a Work in Progress state and is not fully usable. However, it can already be used for testing purposes.
To start, first make sure Rust is installed (this includes `cargo`):

```bash
cargo --version
```

If you get a `command not found` error, either use the offical rustup [install script](https://rustup.rs/) or your distro's package manager to install the Rust toolchain.

Once you're sure `cargo` is up and running, clone this repository:

```bash
git clone https://github.com/ironcapitaleu/arkad.git
cd arkad
```

A few simple examples to test the SEC state machines are already present in `sec/bin/main.rs`. Run them using:

```bash
cargo run
```
