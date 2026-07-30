#![no_std]
#![no_main]

use async_in_embedded::blocking_clock;
use async_in_embedded::utils::{parse_bool, parse_u32};
use chrono::NaiveDateTime;
use cortex_m::delay::Delay;
use defmt::info;
use defmt_rtt as _;
use panic_probe as _;
use stm32f4xx_hal::{
    gpio::GpioExt,
    i2c::{I2cExt, Mode},
    prelude::*,
};

const INIT_DATE_TIME: Option<&str> = option_env!("INIT_DATE_TIME");

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Starting initialization");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut internal_led = gpioa.pa5.into_push_pull_output();
    let mut delay = Delay::new(cp.SYST, rcc.clocks.sysclk().raw());

    info!("Initializing I2C");
    let mut i2c = dp
        .I2C1
        .i2c((gpiob.pb6, gpiob.pb7), Mode::standard(100.kHz()), &mut rcc);
    if let Some(serialized_date_time) = INIT_DATE_TIME {
        info!("Setting Clock to {}", serialized_date_time);
        let configured_datetime = serialized_date_time
            .parse::<NaiveDateTime>()
            .unwrap_or_else(|_| panic!("Could not parse date time {}", serialized_date_time));
        blocking_clock::write(&mut i2c, &configured_datetime);
    }

    loop {
        let clock_time = blocking_clock::read(&mut i2c);
        info!("Current date time {:?}", defmt::Display2Format(&clock_time));
        internal_led.toggle();
        delay.delay_ms(1000);
    }
}
