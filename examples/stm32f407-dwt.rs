//! This example shows how to use the DWT (Data Watchpoint and Trace) unit available
//! on some ARM Cortex-M microcontrollers for more accurate data timings.
//!
//! This is especially useful in applications where (potentially long) interrupt handlers
//! may run while an acquisition is in progress. This may lead to incorrect bit timings,
//! resulting in timeouts and/or CRC errors.

#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::{prelude::*, stm32};
use stm32f4xx_hal as hal;

use dht11::Dht11;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Enable the DWT counter. This *needs* to be done manually.
    cp.DWT.enable_cycle_counter();

    // The DATA pin is connected to PE2.
    let gpio = dp.GPIOE.split();
    let pin = gpio.pe2.into_open_drain_output();

    // Create a delay abstraction based on SysTick.
    // We are using the HSE oscillator here for accurate delays.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    // Create an instance of our device
    let mut dht11 = Dht11::new(pin);

    loop {
        hprintln!("{:?}", dht11.perform_measurement(&mut delay)).unwrap();
        delay.delay_ms(1000_u16);
    }
}
