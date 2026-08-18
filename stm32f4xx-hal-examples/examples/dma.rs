#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use defmt::{error, info, trace};
use defmt_rtt as _;
use fugit::Duration;
use panic_probe as _;
use stm32f4xx_hal::dma::config::DmaConfig;
use stm32f4xx_hal::dma::{PeripheralToMemory, StreamsTuple, Transfer};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::serial::Tx;
use stm32f4xx_hal::timer::{CounterMs, Event};
use stm32f4xx_hal::{pac, prelude::*};
use stm32f4xx_hal_examples::statistics::Statistics;
use stm32f4xx_hal_examples::writer::Buffer;

use core::fmt::Write;

const UART_BUFFER_SIZE: usize = 1024;

const SEND_PERIOD: Duration<u32, 1, 1000> = fugit::MillisDurationU32::millis(1000);

static STATISTICS: Mutex<RefCell<Statistics>> = Mutex::new(RefCell::new(Statistics::new()));
static SEND_TIMER: Mutex<RefCell<Option<CounterMs<pac::TIM2>>>> = Mutex::new(RefCell::new(None));
static SEND_BUFFER: Mutex<RefCell<Buffer<128>>> = Mutex::new(RefCell::new(Buffer::new()));

static RX_TRANSFER: Mutex<
    RefCell<
        Option<
            Transfer<
                stm32f4xx_hal::dma::Stream5<stm32f4xx_hal::pac::DMA1>,
                4,
                stm32f4xx_hal::serial::Rx<USART2>,
                PeripheralToMemory,
                &mut [u8; UART_BUFFER_SIZE],
            >,
        >,
    >,
> = Mutex::new(RefCell::new(None));

static IDLE_RX_BUFFER: Mutex<RefCell<Option<&mut [u8; UART_BUFFER_SIZE]>>> =
    Mutex::new(RefCell::new(None));
static UART_TX: Mutex<RefCell<Option<Tx<pac::USART2>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn USART2() {
    trace!("USART2 interrupt is triggered");
    cortex_m::interrupt::free(|cs| {
        if let Some(transfer) = RX_TRANSFER.borrow(cs).borrow_mut().as_mut() {
            if transfer.is_idle() {
                let new_buffer = IDLE_RX_BUFFER.borrow(cs).take().unwrap();
                let filled_num = UART_BUFFER_SIZE - transfer.number_of_transfers() as usize;
                match transfer.next_transfer(new_buffer) {
                    Ok((buf, _)) => {
                        let mut stats = STATISTICS.borrow(cs).borrow_mut();
                        buf[..filled_num]
                            .iter()
                            .for_each(|el| stats.update_statistics(*el));
                        buf.fill(0);
                        IDLE_RX_BUFFER.borrow(cs).replace(Some(buf));

                        let mut buffer = SEND_BUFFER.borrow(cs).borrow_mut();
                        if buffer.is_read_finished() {
                            writeln!(buffer, "{}", stats).unwrap();
                            UART_TX.borrow(cs).borrow_mut().as_mut().unwrap().listen();
                        }
                    }
                    Err(err) => {
                        error!("Error USART2 {:?}", defmt::Debug2Format(&err));
                        match err {
                            stm32f4xx_hal::dma::DMAError::NotReady(b)
                            | stm32f4xx_hal::dma::DMAError::SmallBuffer(b)
                            | stm32f4xx_hal::dma::DMAError::Overrun(b) => {
                                IDLE_RX_BUFFER.borrow(cs).replace(Some(b));
                            }
                        }
                    }
                }
            }
            transfer.clear_idle_interrupt();
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
fn DMA1_STREAM5() {
    trace!("DMA1_STREAM5 interrupt is triggered");
    cortex_m::interrupt::free(|cs| {
        if let Some(transfer) = RX_TRANSFER.borrow(cs).borrow_mut().as_mut() {
            if transfer.is_transfer_complete() {
                let new_buffer = IDLE_RX_BUFFER.borrow(cs).take().unwrap();
                match transfer.next_transfer(new_buffer) {
                    Ok((buf, _)) => {
                        let mut stats = STATISTICS.borrow(cs).borrow_mut();
                        buf.iter().for_each(|el| stats.update_statistics(*el));
                        buf.fill(0);
                        IDLE_RX_BUFFER.borrow(cs).replace(Some(buf));

                        let mut buffer = SEND_BUFFER.borrow(cs).borrow_mut();
                        if buffer.is_read_finished() {
                            writeln!(buffer, "{}", stats).unwrap();
                            UART_TX.borrow(cs).borrow_mut().as_mut().unwrap().listen();
                        }
                    }
                    Err(err) => {
                        error!("Error DMA1_STREAM5 {:?}", defmt::Debug2Format(&err));
                    }
                }
            }
            transfer.clear_transfer_complete();
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
    info!("DMA INTERRUPT EXAMPLE");
    info!("Starting initialization");
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let gpioa = dp.GPIOA.split(&mut rcc);

    info!("Initializing USART2");
    let (uart_tx, mut uart_rx) = dp
        .USART2
        .serial::<u8>(
            (gpioa.pa2, gpioa.pa3),
            stm32f4xx_hal::serial::Config::default()
                .dma(stm32f4xx_hal::serial::config::DmaConfig::Rx),
            &mut rcc,
        )
        .unwrap()
        .split();
    uart_rx.listen_idle();

    let rx_buffer =
        cortex_m::singleton!(RX_BUFFER_1: [u8; UART_BUFFER_SIZE] = [0;UART_BUFFER_SIZE]).unwrap();
    let idle_rx_buffer =
        cortex_m::singleton!(IDLE_RX_BUFFER: [u8; UART_BUFFER_SIZE] = [0;UART_BUFFER_SIZE])
            .unwrap();
    let rx_stream = StreamsTuple::new(dp.DMA1, &mut rcc).5;

    let mut rx_transfer = Transfer::init_peripheral_to_memory(
        rx_stream,
        uart_rx,
        rx_buffer,
        None,
        DmaConfig::default()
            .memory_increment(true)
            .transfer_complete_interrupt(true),
    );

    rx_transfer.start(|_| {});

    let mut log_timer = dp.TIM2.counter_ms(&mut rcc);
    log_timer.start(SEND_PERIOD).unwrap();
    log_timer.listen(Event::Update);

    cortex_m::interrupt::free(|cs| {
        IDLE_RX_BUFFER.borrow(cs).replace(Some(idle_rx_buffer));
        RX_TRANSFER.borrow(cs).replace(Some(rx_transfer));
        UART_TX.borrow(cs).replace(Some(uart_tx));
        SEND_TIMER.borrow(cs).replace(Some(log_timer));
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART2);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA1_STREAM5);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }

    loop {
        asm::wfi();
    }
}
