#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, delay::Delay, peripherals::Peripherals, prelude::*, rng::Rng,
    system::SystemControl, timer::systimer::SystemTimer,
};
use esp_println::println;
use esp_wifi::{wifi, wifi_interface::WifiStack};
use smoltcp::iface::SocketStorage;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

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
    let (iface, device, mut controller, sockets) = wifi::utils::create_network_interface(
        &wifi_init,
        peripherals.WIFI,
        wifi::WifiStaDevice,
        &mut socket_set_entries,
    )
    .unwrap();

    let client_congig = wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method: wifi::AuthMethod::WPA2Personal,
        ..Default::default()
    });
    controller.set_configuration(&client_congig).unwrap();

    controller.start().unwrap();
    controller.connect().unwrap();

    println!("Waiting to get connected...");
    loop {
        match controller.is_connected() {
            Ok(connected) => {
                if connected {
                    println!("Wifi connected: {:?}", connected);
                    break;
                }
            }
            Err(err) => {
                println!("Got connection error: {:?}", err);
                delay.delay(500.millis());
            }
        }
    }

    let wifi_stack = WifiStack::new(iface, device, sockets, esp_wifi::current_millis);
    println!("Waiting for IP address...");
    loop {
        wifi_stack.work();

        if wifi_stack.is_iface_up() {
            println!("Got IP: {:?}", wifi_stack.get_ip_info());
            break;
        }
        delay.delay_millis(500);
    }

    panic!()
}
