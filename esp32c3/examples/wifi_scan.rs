#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, rng::Rng, system::SystemControl,
    timer::systimer::SystemTimer,
};
use esp_println::println;
use esp_wifi::wifi;
use smoltcp::iface::SocketStorage;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let (_, _, mut controller, _) = wifi::utils::create_network_interface(
        &wifi_init,
        peripherals.WIFI,
        wifi::WifiStaDevice,
        &mut socket_set_entries,
    )
    .unwrap();

    controller.start().unwrap();

    let res: Result<(heapless::Vec<wifi::AccessPointInfo, 10>, usize), wifi::WifiError> =
        controller.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    panic!()
}
