#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::{debug, info};
use defmt_rtt as _;
use fugit::Duration;
use panic_probe as _;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal_examples::{statistics::Statistics, writer};

const SEND_PERIOD: Duration<u32, 1, 1000> = fugit::MillisDurationU32::millis(1000);

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("BLOCKING EXAMPLE - DO NOT USE");
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
    timer.start(u32::MAX.millis()).unwrap();

    let mut statistics = Statistics::new();
    info!("Started data collection");
    let mut next_send = timer.now() + SEND_PERIOD;
    loop {
        let read_byte = embedded_hal_nb::nb::block!(uart_rx.read()).unwrap();
        statistics.update_statistics(read_byte);
        writeln!(uart_tx, "{}", statistics).unwrap();

        // sending here would starve on a blocking read
        // let now = timer.now();
        // if now > next_send {
        //     debug!(
        //         "Statistics: {:?}, seconds from the start: {}",
        //         defmt::Display2Format(&statistics),
        //         now.duration_since_epoch().to_secs()
        //     );
        //     writeln!(uart_tx, "{}", statistics).unwrap();
        //     next_send += SEND_PERIOD;
        // }
    }
}
