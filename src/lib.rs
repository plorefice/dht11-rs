//! Platform-agnostic Rust driver for the DHT11 temperature and humidity sensor,
//! using [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) traits.

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};

/// How long to wait for a pulse on the data line (in microseconds).
const TIMEOUT_US: u16 = 1_000;

/// Error type for this crate.
#[derive(Debug)]
pub enum Error<E> {
    /// Timeout during communication.
    Timeout,
    /// CRC mismatch.
    CrcMismatch,
    /// GPIO error.
    Gpio(E),
}

/// A DHT11 device.
pub struct Dht11<GPIO> {
    /// The concrete GPIO pin implementation.
    gpio: GPIO,
}

/// Results of a reading performed by the DHT11.
#[derive(Debug)]
pub struct Measurement {
    /// The measured temperature.
    pub temperature: f32,
    /// The measured humidity.
    pub humidity: f32,
}

impl<GPIO, E> Dht11<GPIO>
where
    GPIO: InputPin<Error = E> + OutputPin<Error = E>,
{
    /// Creates a new DHT11 device connected to the specified pin.
    pub fn new(gpio: GPIO) -> Self {
        Dht11 { gpio }
    }

    /// Destroys the driver, returning the GPIO instance.
    pub fn destroy(self) -> GPIO {
        self.gpio
    }

    /// Performs a reading of the sensor.
    pub fn perform_measurement<D>(&mut self, delay: &mut D) -> Result<Measurement, Error<E>>
    where
        D: DelayUs<u16> + DelayMs<u16>,
    {
        let mut data = [0u8; 5];

        // Perform initial handshake
        self.perform_handshake(delay)?;

        // Read bits
        for i in 0..40 {
            data[i / 8] <<= 1;
            if self.read_bit(delay)? {
                data[i / 8] |= 1;
            }
        }

        // Finally wait for line to go idle again.
        self.wait_for_pulse(true, delay)?;

        // Check CRC
        let crc = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if crc != data[4] {
            return Err(Error::CrcMismatch);
        }

        // Compute temperature
        let mut temp = (data[2] & 0x7f) as f32 + data[3] as f32 * 0.1;
        if data[2] & 0x80 != 0 {
            temp = -temp;
        }

        Ok(Measurement {
            temperature: temp,
            humidity: data[0] as f32 + data[1] as f32 * 0.1,
        })
    }

    fn perform_handshake<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayUs<u16> + DelayMs<u16>,
    {
        // Set pin as floating to let pull-up raise the line and start the reading process.
        self.set_input()?;
        delay.delay_ms(1);

        // Pull line low for at least 18ms to send a start command.
        self.set_low()?;
        delay.delay_ms(20);

        // Restore floating
        self.set_input()?;
        delay.delay_us(40);

        // As a response, the device pulls the line low for 80us and then high for 80us.
        self.read_bit(delay)?;

        Ok(())
    }

    fn read_bit<D>(&mut self, delay: &mut D) -> Result<bool, Error<E>>
    where
        D: DelayUs<u16> + DelayMs<u16>,
    {
        let low = self.wait_for_pulse(true, delay)?;
        let high = self.wait_for_pulse(false, delay)?;
        Ok(high > low)
    }

    fn wait_for_pulse<D>(&mut self, level: bool, delay: &mut D) -> Result<u16, Error<E>>
    where
        D: DelayUs<u16> + DelayMs<u16>,
    {
        let mut count = 0;

        while self.read_line()? != level {
            count += 1;
            if count > TIMEOUT_US {
                return Err(Error::Timeout);
            }
            delay.delay_us(1);
        }

        Ok(count)
    }

    fn set_input(&mut self) -> Result<(), Error<E>> {
        self.gpio.set_high().map_err(Error::Gpio)
    }

    fn set_low(&mut self) -> Result<(), Error<E>> {
        self.gpio.set_low().map_err(Error::Gpio)
    }

    fn read_line(&self) -> Result<bool, Error<E>> {
        self.gpio.is_high().map_err(Error::Gpio)
    }
}
