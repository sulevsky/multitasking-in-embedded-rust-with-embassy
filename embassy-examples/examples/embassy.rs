#![no_std]
#![no_main]

use core::fmt::Write as _;
use defmt::info;
use defmt_rtt as _;
use embassy_examples::statistics::Statistics;
use embassy_examples::writer::Buffer;
use embassy_executor::Spawner;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::{UartRx, UartTx};
use embassy_stm32::{bind_interrupts, dma, peripherals, usart};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use panic_probe as _;

const SEND_PERIOD: Duration = Duration::from_millis(1000);

static STATISTICS: Mutex<CriticalSectionRawMutex, Statistics> = Mutex::new(Statistics::new());
static UART_TX: Mutex<CriticalSectionRawMutex, Option<UartTx<'static, Async>>> = Mutex::new(None);
static STATISTICS_UPDATED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(
    struct Irqs {
        USART2 => usart::InterruptHandler<peripherals::USART2>;
        DMA1_STREAM5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
        DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
    }
);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("EMBASSY EXAMPLE");
    info!("Starting initialization");
    let p = embassy_stm32::init(Default::default());
    info!("Initializing USART2");
    let (uart_tx, uart_rx) = embassy_stm32::usart::Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
        p.DMA1_CH6,
        p.DMA1_CH5,
        Irqs,
        embassy_stm32::usart::Config::default(),
    )
    .unwrap()
    .split();
    UART_TX.lock().await.replace(uart_tx);
    spawner.spawn(collect_statistics(uart_rx).unwrap());
    spawner.spawn(send_statistics_periodically().unwrap());
    spawner.spawn(send_statistics_on_update().unwrap());
}

#[embassy_executor::task]
async fn collect_statistics(mut uart_rx: UartRx<'static, Async>) {
    let mut buffer = [0u8; 512];
    loop {
        let num_bytes_read = uart_rx.read_until_idle(&mut buffer).await.unwrap();
        for i in 0..num_bytes_read {
            STATISTICS.lock().await.update_statistics(buffer[i]);
        }
        STATISTICS_UPDATED.signal(());
        buffer.fill(0);
    }
}

#[embassy_executor::task]
async fn send_statistics_on_update() {
    loop {
        STATISTICS_UPDATED.wait().await;
        send_statistics().await;
    }
}

#[embassy_executor::task]
async fn send_statistics_periodically() {
    loop {
        send_statistics().await;
        Timer::after(SEND_PERIOD).await;
    }
}

async fn send_statistics() {
    let mut buffer: Buffer<128> = Buffer::new();
    writeln!(buffer, "{}", STATISTICS.lock().await).unwrap();
    let mut tx = UART_TX.lock().await;
    tx.as_mut().unwrap().write(buffer.as_bytes()).await.unwrap();
}
