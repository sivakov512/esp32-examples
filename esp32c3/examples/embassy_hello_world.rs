#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_println::println;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    spawner.spawn(counter()).unwrap();

    let mut count = 0;
    loop {
        println!("Main task count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[embassy_executor::task]
async fn counter() {
    let mut count = 0;
    loop {
        println!("Spawn task count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(1_000)).await;
    }
}
