#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    system::SystemControl,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_println::println;
use esp_wifi::{
    wifi::{self, WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState},
    EspWifiInitFor,
};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;

    let wifi_init = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let (wifi_iface, controller) =
        wifi::new_with_mode(&wifi_init, peripherals.WIFI, WifiStaDevice).unwrap();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    let dhcp_config = embassy_net::Config::dhcpv4(Default::default());
    let net_stack = &*mk_static!(
        embassy_net::Stack<WifiDevice<'_, WifiStaDevice>>,
        embassy_net::Stack::new(
            wifi_iface,
            dhcp_config,
            mk_static!(
                embassy_net::StackResources<3>,
                embassy_net::StackResources::<3>::new()
            ),
            1234,
        )
    );

    spawner.spawn(run_net_stack(&net_stack)).unwrap();
    spawner.spawn(connect(controller)).unwrap();

    loop {
        if net_stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = net_stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn run_net_stack(net_stack: &'static embassy_net::Stack<WifiDevice<'static, WifiStaDevice>>) {
    net_stack.run().await;
}

#[embassy_executor::task]
async fn connect(mut controller: WifiController<'static>) {
    loop {
        match wifi::get_wifi_state() {
            WifiState::StaConnected => {
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5_000)).await;
            }
            _ => {}
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let config = wifi::Configuration::Client(wifi::ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                auth_method: wifi::AuthMethod::WPA2Personal,
                ..Default::default()
            });
            controller.set_configuration(&config).unwrap();
            controller.start().await.unwrap();
        }
        match controller.connect().await {
            Ok(_) => println!("CONNECTED"),
            Err(e) => {
                println!("FAILED TO CONNECT: {e:?}");
                Timer::after(Duration::from_millis(500)).await;
            }
        }
    }
}
