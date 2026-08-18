#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::{error, info};
use defmt_rtt as _;
use fugit::Duration;
use panic_probe as _;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal_examples::{statistics::Statistics, writer::Buffer};

const SEND_PERIOD: Duration<u32, 1, 1000> = fugit::MillisDurationU32::millis(1000);

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("POLLING EXAMPLE");
    info!("Starting initialization");
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let gpioa = dp.GPIOA.split(&mut rcc);

    info!("Initializing USART2");
    let (mut uart_tx, mut uart_rx) = dp
        .USART2
        .serial::<u8>(
            (gpioa.pa2, gpioa.pa3),
            stm32f4xx_hal::serial::Config::default(),
            &mut rcc,
        )
        .unwrap()
        .split();

    let mut timer = dp.TIM2.counter_ms(&mut rcc);
    timer.start(u32::MAX.micros()).unwrap();

    let mut statistics = Statistics::new();
    let mut send_buffer = Buffer::<128>::new();
    info!("Started data collection");
    let mut next_send = timer.now() + SEND_PERIOD;
    loop {
        match uart_rx.read() {
            Ok(byte) => {
                statistics.update_statistics(byte);
                if send_buffer.is_read_finished() {
                    writeln!(send_buffer, "{}", statistics).unwrap();
                }
            }
            Err(stm32f4xx_hal::nb::Error::Other(error)) => {
                error!("UART error: {:?}", defmt::Debug2Format(&error));
            }
            Err(stm32f4xx_hal::nb::Error::WouldBlock) => {
                // requires blocking to complete
            }
        }

        let now = timer.now();
        if now > next_send {
            if send_buffer.is_read_finished() {
                writeln!(send_buffer, "{}", statistics).unwrap();
            }
            next_send += SEND_PERIOD;
        }
        if !send_buffer.is_read_finished() {
            if let Some(byte) = send_buffer.read_next() {
                match uart_tx.write(byte) {
                    Ok(_) => send_buffer.mark_byte_as_read(),
                    Err(stm32f4xx_hal::nb::Error::Other(error)) => {
                        error!("UART error: {:?}", defmt::Debug2Format(&error));
                    }
                    Err(stm32f4xx_hal::nb::Error::WouldBlock) => {
                        // requires blocking to complete
                    }
                }
            }
        }
    }
}
