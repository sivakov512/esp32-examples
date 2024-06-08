#![no_std]
#![no_main]

use core::{fmt::Write, str::from_utf8};
use embedded_storage::{ReadStorage, Storage};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    uart::{self, Uart},
};
use esp_storage::FlashStorage;

const MEM_OFFSET: u32 = 0x9000;
const MEMORY_LIMIT: usize = u8::max_value() as usize;

const VALUE2WRITE: &str = env!("VALUE2WRITE");

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    let mut storage = FlashStorage::new();

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

    storage
        .write(MEM_OFFSET, &[VALUE2WRITE.len() as u8])
        .unwrap();
    storage
        .write(MEM_OFFSET + 1, VALUE2WRITE.as_bytes())
        .unwrap();

    loop {
        let mut header = [0; 1];
        storage.read(MEM_OFFSET, &mut header).unwrap();
        let len2read = header[0] as usize;

        let mut read = [0; MEMORY_LIMIT];
        let slice2read = &mut read[..len2read];
        storage.read(MEM_OFFSET + 1, &mut *slice2read).unwrap();

        write!(uart, "{}\r\n", from_utf8(&slice2read).unwrap()).unwrap();

        delay.delay_millis(1_000);
    }
}
