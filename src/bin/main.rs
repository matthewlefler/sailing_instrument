#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;

use esp_hal::{
    gpio::Level,
    main,
    rmt::{Rmt, TxChannelConfig, TxChannelCreator},
    time::Rate,
};

esp_app_desc!();

mod argb;
mod gps;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // RMT clock = 80 MHz.
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).expect("RMT init failed");

    // Divide 80 MHz by 2 => 40 MHz.
    // One RMT tick = 25 ns.
    let tx_config = TxChannelConfig::default()
        .with_clk_divider(2)
        .with_idle_output_level(Level::Low);

    let mut rgb_led_channel = rmt
        .channel0
        .configure_tx(&tx_config)
        .expect("RMT TX init failed")
        .with_pin(peripherals.GPIO38); // argb pin on the esp32-s3 DevKitC-1

    let gps_tx_pin = peripherals.GPIO17;
    let gps_rx_pin = peripherals.GPIO18;

    gps::gps_init(gps_tx_pin, gps_rx_pin, peripherals.UART1);

    loop {
        if let Some(fix) = gps::main_loop() {
			if fix {
            	rgb_led_channel = argb::write_color(rgb_led_channel, 0x0, 0x10, 0x0);
			} else {
				rgb_led_channel = argb::write_color(rgb_led_channel, 0x10, 0x0, 0x0);
			}
        }


    }
}
