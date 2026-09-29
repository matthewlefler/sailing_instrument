use esp_hal::{
    Blocking, 
    gpio::Level, 
    rmt::{Channel, PulseCode, Tx},
};
use core::iter::Iterator;
//
// Create the 24 WS2812 bits.
//
// WS2812 expects:
//
//     G R B
//
// rather than:
//
//     R G B
//
pub fn write_color(
    channel: Channel<'_, Blocking, Tx>,
    red: u8,
    green: u8,
    blue: u8,
) -> Channel<'_, Blocking, Tx> {
    let mut data = [PulseCode::end_marker(); 25];

    let bytes = [
        green,
        red,
        blue,
    ];

    let mut index = 0;

    for byte in bytes {
        for bit in (0..8).rev() {
            let one = (byte & (1 << bit)) != 0;

            if one {
                // Logical 1:
                //
                // HIGH ~0.8 us
                // LOW  ~0.45 us
                //
                // At 40 MHz:
                // 32 * 25ns = 800ns
                // 18 * 25ns = 450ns

                data[index] = PulseCode::new(
                    Level::High,
                    32,
                    Level::Low,
                    18,
                );
            } else {
                // Logical 0:
                //
                // HIGH ~0.4 us
                // LOW  ~0.85 us
                //
                // 16 * 25ns = 400ns
                // 34 * 25ns = 850ns

                data[index] = PulseCode::new(
                    Level::High,
                    16,
                    Level::Low,
                    34,
                );
            }

            index += 1;
        }
    }

    data[24] = PulseCode::end_marker();

    let transaction = channel
        .transmit(&data)
        .expect("RMT transmit failed");

    transaction
        .wait()
        .expect("RMT wait failed")
}
