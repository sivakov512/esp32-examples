#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio,
    peripherals::{Peripherals, UART1},
    prelude::*,
    system::SystemControl,
    timer::timg::TimerGroup,
    uart::{self, Uart, UartRx, UartTx},
};
use esp_println::println;

const BUF_SIZE: usize = 1;
static DATAPIPE: Pipe<CriticalSectionRawMutex, BUF_SIZE> = Pipe::new();

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    let io = gpio::Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins = uart::TxRxPins::new_tx_rx(io.pins.gpio8, io.pins.gpio9);

    let mut uart = Uart::new_async_with_config(
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
    );
    uart.set_rx_fifo_full_threshold(BUF_SIZE as u16).unwrap();
    let (tx, rx) = uart.split();

    spawner.spawn(reader(rx)).unwrap();
    spawner.spawn(writer(tx)).unwrap();
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, UART1, esp_hal::Async>) {
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        let r = rx.read_async(&mut buf).await;

        match r {
            Ok(_) => {
                DATAPIPE.write_all(&buf).await;
            }
            Err(e) => println!("RX error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn writer(mut tx: UartTx<'static, UART1, esp_hal::Async>) {
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        DATAPIPE.read(&mut buf).await;
        match tx.write_async(&buf).await {
            Ok(_) => {
                tx.flush_async().await.unwrap();
            }
            Err(e) => println!("TX error: {:?}", e),
        }
    }
}
