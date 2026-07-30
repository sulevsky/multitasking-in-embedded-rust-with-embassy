use stm32f4xx_hal::{adc::config::AdcConfig, gpio::Analog};

pub struct Adc {
    adc: stm32f4xx_hal::adc::Adc<stm32f4xx_hal::pac::ADC1>,
}

impl Adc {
    pub fn new(
        dp_adc: stm32f4xx_hal::pac::ADC1,
        rcc: &mut stm32f4xx_hal::rcc::Rcc,
        adc_pin: &stm32f4xx_hal::gpio::Pin<'A', 0, Analog>,
    ) -> Self {
        let adc_config = AdcConfig::default()
            .clock(stm32f4xx_hal::adc::config::Clock::Pclk2_div_4)
            .continuous(stm32f4xx_hal::adc::config::Continuous::Continuous);
        let mut adc = stm32f4xx_hal::adc::Adc::new(dp_adc, true, adc_config, rcc);
        adc.configure_channel(
            adc_pin,
            stm32f4xx_hal::adc::config::Sequence::One,
            stm32f4xx_hal::adc::config::SampleTime::Cycles_480,
        );
        adc.start_conversion();

        Self { adc }
    }
    pub fn measure(&self) -> u16 {
        self.adc.current_sample()
    }
}
