#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{
	delay::Delay,
	gpio::{Level, Output, OutputConfig},
	rmt::{Rmt, PulseCode, TxChannelConfig, TxChannelCreator},
	time::{Rate},
	main,
};
// Required by the ESP-IDF bootloader
esp_app_desc!();
#[main]
fn main() -> ! {
	let peripherals = esp_hal::init(esp_hal::Config::default());

	// Configure frequency based on chip type
	let freq = Rate::from_mhz(80);
	let rmt = Rmt::new(peripherals.RMT, freq).expect("wha");

	let tx_config = TxChannelConfig::default().with_clk_divider(255);

	let mut channel = rmt.channel0.configure_tx(&tx_config).expect("wha 2");

	let delay = Delay::new();

	let mut data = [PulseCode::new(Level::High, 200, Level::Low, 50); 20];
	data[data.len() - 2] = PulseCode::new(Level::High, 3000, Level::Low, 500);
	data[data.len() - 1] = PulseCode::end_marker();

	loop {
		let transaction = channel.transmit(&data).expect("whaha");
		channel = transaction.wait().expect("wah 4");
		delay.delay_millis(500);
	}
}