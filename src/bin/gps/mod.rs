use core::cell::RefCell;

use critical_section::Mutex;
use esp_hal::{
    Blocking, gpio::{Input, interconnect::PeripheralOutput}, handler, ram, uart::{Config, DataBits, Instance, Parity, RxConfig, StopBits, Uart, UartInterrupt},
};
use esp_println::{print, println};

mod parsing;

static GPS_COMMAND_LEN: usize = 256;
#[derive(Copy, Clone)]
struct GpsSentence {
    data: [u8; GPS_COMMAND_LEN],
    len: usize,
}

impl GpsSentence {
    const fn empty() -> Self {
        Self {
            data: [0; GPS_COMMAND_LEN],
            len: 0,
        }
    }
}

// https://cdn-shop.adafruit.com/datasheets/PMTK_A11.pdf
pub static SERIAL: Mutex<RefCell<Option<Uart<Blocking>>>> = Mutex::new(RefCell::new(None));

pub fn gps_init(tx_pin: impl esp_hal::gpio::OutputPin + 'static, rx_pin: impl esp_hal::gpio::InputPin + 'static, uart: impl Instance + 'static) -> () {
    let gps_uart_config = Config::default()
		.with_baudrate(9600)
		.with_data_bits(DataBits::_8)
		.with_parity(Parity::None)
		.with_stop_bits(StopBits::_1)
		.with_rx(
			RxConfig::default()
				.with_fifo_full_threshold(GPS_FIFO_FULL)
				.with_timeout(2),
		);

	let mut gps_uart = Uart::new(uart, gps_uart_config)
		.expect("Failed to initialize UART")
		.with_tx(tx_pin)
		.with_rx(rx_pin);

	gps_uart.set_interrupt_handler(gps_handler);

	critical_section::with(|cs| {
		gps_uart.clear_interrupts(
			UartInterrupt::RxFifoFull |
			UartInterrupt::RxTimeout
		);

		gps_uart.listen(
			UartInterrupt::RxFifoFull |
			UartInterrupt::RxTimeout
		);

		SERIAL.borrow_ref_mut(cs).replace(gps_uart);
	});
}

pub static GPS_FIFO_FULL: u16 = 32;
static GPS_STRING_BUFFER: Mutex<RefCell<[u8; 256]>> =
    Mutex::new(RefCell::new([0; GPS_COMMAND_LEN]));
static GPS_STRING_BUFFER_INDEX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));

//
static GPS_COMMAND_QUEUE_SIZE: usize = 5;
static GPS_COMMAND_QUEUE_LEN: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0)); // mod GPS_COMMAND_QUEUE_SIZE
static GPS_COMMAND_QUEUE: Mutex<RefCell<[GpsSentence; GPS_COMMAND_QUEUE_SIZE]>> =
    Mutex::new(RefCell::new([GpsSentence::empty(); GPS_COMMAND_QUEUE_SIZE]));

struct GGACommand {
    utc_time: u32,
    latitude: i32,
    longitude: i32,
    // 0 Fix not available
    // 1 GPS fix
    // 2 Differential GPS fix``
    fix_indicator: u8,
    satilites_used: u8,                    // 0 to 14
    horizontal_dilution_of_precision: u16, // 0 to 1000 (0.00 to 10.00)
    antenna_altitude: u32,
    age_of_diff_correction: Option<u16>,
}
struct GSACommand {
    mode: u8, // M (manual) or A (automatic)
    fix: u8, // 1 (no fix), 2 (2D), 3 (3D)
    // for ranges 1-32 see TODO: idk
    // for ranges from 193-195 see: https://en.wikipedia.org/wiki/Quasi-Zenith_Satellite_System
    satilites_used: [u8; 12],
    position_dilution_of_precision: u16,
    horizontal_dilution_of_precision: u16,
    vertical_dilution_of_precision: u16,
}

#[derive(Copy, Clone)]
struct Satellite {
    satellite_id: u8, // 1 - 32, or 193 - 195
    elevation: u8, // 0 - 90 degrees
    azimuth: u8, // 0 - 359 degrees
    signal_to_noise_ratio: Option<u8>, // 0-99 otherwise null if not tracking
}
struct GSVCommand {
    number_of_messages: u8, // 1 to 4
    message_number: u8, // 1 to 4
    satellites_in_view: u8, // up to 99
    satellites: [Satellite; 4]
}
struct RMCCommand {
    utc_time: u32,
    status: u8, // A (valid) or V (invalid)
    latitude: i32,
    longitude: i32,
    knots: u16,
    course: u16, // true heading
    date: u32,
    magnetic_variation: Option<i16>,
    mode: u8 // A (autonomous) or D (differental) or E (estimated) mode
}
struct VTGCommand {
    course: u16,
    knots: u16,
    mode: u8 // A (autonomous) or D (differental) or E (estimated) mode
}
enum GpsCommand {
    GGA(GGACommand),
    GSA(GSACommand),
    GSV(GSVCommand),
    RMC(RMCCommand),
    VTG(VTGCommand),
}

#[handler]
#[ram]
pub fn gps_handler() {
    critical_section::with(|cs| {
        let mut serial = SERIAL.borrow_ref_mut(cs);

        if let Some(serial) = serial.as_mut() {
            let mut buf = [0u8; GPS_FIFO_FULL as usize];

            if let Ok(count) = serial.read_buffered(&mut buf) {
                let mut buffer = GPS_STRING_BUFFER.borrow_ref_mut(cs);
                let mut buffer_len = GPS_STRING_BUFFER_INDEX.borrow_ref_mut(cs);
                for &byte in &buf[..count] {
                    match byte {
                        b'\r' => {
                            let mut static_gps_command_queue = GPS_COMMAND_QUEUE.borrow_ref_mut(cs);
                            let mut gps_command_queue_len =
                                GPS_COMMAND_QUEUE_LEN.borrow_ref_mut(cs);

                            static_gps_command_queue[*gps_command_queue_len] = GpsSentence {
                                data: *buffer,
                                len: *buffer_len,
                            };
                            *gps_command_queue_len += 1;

                            *buffer_len = 0;
                        }
                        b'\n' => {}
                        _ => {
                            buffer[*buffer_len] = byte;
                            *buffer_len += 1;
                        }
                    }
                }
            }

            serial.clear_interrupts(UartInterrupt::RxFifoFull | UartInterrupt::RxTimeout);
        }
    });
}

pub fn main_loop() -> Option<bool> {
    // local buffer
    let mut local_gps_command_queue: [GpsSentence; GPS_COMMAND_QUEUE_SIZE] =
        [GpsSentence::empty(); GPS_COMMAND_QUEUE_SIZE];
    let mut local_gps_command_queue_len = 0;

    // get cmds if exists
    critical_section::with(|cs| {
        let mut gps_command_queue_len = GPS_COMMAND_QUEUE_LEN.borrow_ref_mut(cs);
        local_gps_command_queue_len = *gps_command_queue_len;

        if local_gps_command_queue_len > 0 {
            let static_gps_command_queue = GPS_COMMAND_QUEUE.borrow_ref_mut(cs);
            for i in 0..local_gps_command_queue_len {
                local_gps_command_queue[i] = static_gps_command_queue[i];
            }
            *gps_command_queue_len = 0;
        } else {
            // no commands in buffer
            return;
        }
    });

    // parse
    for i in 0..local_gps_command_queue_len {
        let command = local_gps_command_queue[i];

        match parsing::parse_gps_command(command) {
            Ok(cmd) => match cmd {
                GpsCommand::GGA(cmd) => {
                    println!("{} {}", cmd.latitude, cmd.longitude);
                    return Some(cmd.fix_indicator != 0);
                }
                _ => {}
            },
            Err(err) => {
                println!("got gps parse err {:?}", err);
                match err {
                    parsing::GpsParseError::NoFix => return Some(false),
                    _ => {}
                }
            }
        }
    }

    None
}
