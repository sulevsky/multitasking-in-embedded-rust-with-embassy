#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write};

use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use defmt::{error, info, trace};
use defmt_rtt as _;
use fugit::Duration;
use panic_probe as _;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::timer::{CounterMs, Event};
use stm32f4xx_hal_examples::statistics::Statistics;
use stm32f4xx_hal_examples::writer::Buffer;

const SEND_PERIOD: Duration<u32, 1, 1000> = fugit::MillisDurationU32::millis(1000);

static UART_RX: Mutex<RefCell<Option<stm32f4xx_hal::serial::Rx<stm32f4xx_hal::pac::USART2>>>> =
    Mutex::new(RefCell::new(None));
static UART_TX: Mutex<RefCell<Option<stm32f4xx_hal::serial::Tx<stm32f4xx_hal::pac::USART2>>>> =
    Mutex::new(RefCell::new(None));
static STATISTICS: Mutex<RefCell<Statistics>> = Mutex::new(RefCell::new(Statistics::new()));
static SEND_TIMER: Mutex<RefCell<Option<CounterMs<stm32f4xx_hal::pac::TIM2>>>> =
    Mutex::new(RefCell::new(None));
static SEND_BUFFER: Mutex<RefCell<Buffer<128>>> = Mutex::new(RefCell::new(Buffer::new()));

#[interrupt]
fn USART2() {
    trace!("USART2 interrupt is triggered");
    cortex_m::interrupt::free(|cs| {
        if let Some(uart_rx) = UART_RX.borrow(cs).borrow_mut().as_mut() {
            match uart_rx.read() {
                Ok(byte) => {
                    let mut statistics = STATISTICS.borrow(cs).borrow_mut();
                    statistics.update_statistics(byte);
                    let mut buffer = SEND_BUFFER.borrow(cs).borrow_mut();
                    if buffer.is_read_finished() {
                        writeln!(buffer, "{}", statistics).unwrap();
                        UART_TX.borrow(cs).borrow_mut().as_mut().unwrap().listen();
                    }
                }
                Err(stm32f4xx_hal::nb::Error::Other(error)) => {
                    error!("UART error: {:?}", defmt::Debug2Format(&error));
                }
                Err(stm32f4xx_hal::nb::Error::WouldBlock) => {
                    // requires blocking to complete
                }
            }
        }

        if let Some(uart_tx) = UART_TX.borrow(cs).borrow_mut().as_mut() {
            let mut buffer = SEND_BUFFER.borrow(cs).borrow_mut();
            if buffer.is_read_finished() {
                uart_tx.unlisten();
            } else {
                if let Some(byte) = buffer.read_next() {
                    if uart_tx.write(byte).is_ok() {
                        buffer.mark_byte_as_read();
                    }
                }
            }
        }
    });
}

#[interrupt]
fn TIM2() {
    trace!("TIM2 interrupt is triggered");
    cortex_m::interrupt::free(|cs| {
        if let Some(timer) = SEND_TIMER.borrow(cs).borrow_mut().as_mut() {
            timer.clear_all_flags();
        }
        let mut buffer = SEND_BUFFER.borrow(cs).borrow_mut();
        if buffer.is_read_finished() {
            writeln!(buffer, "{}", STATISTICS.borrow(cs).borrow()).unwrap();
            UART_TX.borrow(cs).borrow_mut().as_mut().unwrap().listen();
        }
    });
}

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("INTERRUPT EXAMPLE");
    info!("Starting initialization");
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let gpioa = dp.GPIOA.split(&mut rcc);

    info!("Initializing USART2");
    let (uart_tx, mut uart_rx) = dp
        .USART2
        .serial::<u8>(
            (gpioa.pa2, gpioa.pa3),
            stm32f4xx_hal::serial::Config::default(),
            &mut rcc,
        )
        .unwrap()
        .split();

    uart_rx.listen();

    let mut send_timer = dp.TIM2.counter_ms(&mut rcc);
    send_timer.start(SEND_PERIOD).unwrap();
    send_timer.listen(Event::Update);

    cortex_m::interrupt::free(|cs| {
        UART_RX.borrow(cs).replace(Some(uart_rx));
        UART_TX.borrow(cs).replace(Some(uart_tx));
        SEND_TIMER.borrow(cs).replace(Some(send_timer));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::pac::Interrupt::USART2);
        cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::pac::Interrupt::TIM2);
    }

    loop {
        asm::wfi();
    }
}
