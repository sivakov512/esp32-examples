#![no_std]
#![no_main]

use core::{fmt::Write, str::from_utf8};
use embedded_hal_nb::serial::Write as _;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    uart::{self, Uart},
};
use nb::block;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let io = gpio::Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins = uart::TxRxPins::new_tx_rx(io.pins.gpio8, io.pins.gpio9);

    let mut uart = Uart::new_with_config(
        peripherals.UART1,
        uart::config::Config {
            baudrate: 115_200,
            data_bits: uart::config::DataBits::DataBits8,
            parity: uart::config::Parity::ParityNone,
            stop_bits: uart::config::StopBits::STOP1,
            ..Default::default()
        },
        Some(pins),
        &clocks,
        None,
    );

    loop {
        match block!(uart.read_byte()) {
            Ok(read) => {
                write!(uart, "{}", from_utf8(&[read]).unwrap()).unwrap();
            }
            Err(err) => {
                write!(uart, "Error {:?}", err).unwrap();
            }
        }
        block!(uart.flush()).unwrap();
    }
}
