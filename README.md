# dht11-rs

[![crates.io badge](https://img.shields.io/crates/v/dht11.svg)](https://crates.io/crates/dht11)
[![docs.rs badge](https://docs.rs/dht11/badge.svg)](https://docs.rs/dht11)

Platform-agnostic Rust driver for the DHT11 temperature and humidity sensor,
using [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) traits.

## Requirements

- Rust 1.43+

## Usage

Include library as a dependency in your Cargo.toml

```toml
[dependencies]
dht11 = "0.1.0"
```

```rust
use dht11::Dht11;

// Create an instance of the DHT11 device
let mut dht11 = Dht11::new(pin);

// Perform a sensor reading
let measurement = dht11.perform_measurement(&mut delay).unwrap();
println!("{:?}", measurement);
```

## Examples

See the [examples](examples/) directory for an example on how to use this crate on an STM32F407 MCU.

By default, semihosting is used to display the value of the readings, using OpenOCD or similar.
