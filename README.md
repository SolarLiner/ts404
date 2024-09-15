# Error 404: Screamer not found

**Warning**: This project has been moved back to [`valib`](https://github.com/SolarLiner/valib) in order to facilitate dogfooding the library.

[Download (latest master, unstable)](https://nightly.link/SolarLiner/ts404/workflows/build/master)

A guitar pedal plugin inspired by the most popular screamer pedal.

## Building

### Requirements

- Python
  - Poetry
- Rust (use rustup to get the correct version of the nightly, as defined in the `rust-toolchain` file)
  - `cargo-make`

### Compilation

The following runs all steps of setting up the virtual environment, deriving generated Rust code, and building the
plugins. They will be made available in the `target/bundled` folder.

```shell
cargo make
```
