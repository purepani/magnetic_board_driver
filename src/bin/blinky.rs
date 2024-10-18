#![no_std]
#![no_main]
mod mlx90393;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    flash::Async,
    gpio::{Input, Level, Output, Pin, Pull, Speed},
    i2c, interrupt, peripherals,
    time::hz,
};
use embassy_time::Timer;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::{I2c, Operation};
use mlx90393::MLX90393;
//use embedded_hal::blocking::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let address: u8 = 0x10;
    //let address_write: u8 = 0b0001110;
    //let address_read: u8 = 0b0001111;
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    //let mut led = Output::new(p.PB4, Level::High, Speed::Low);
    //let mut red = Output::new(p.PB7, Level::High, Speed::Low);

    let sda = p.PB1;
    let scl = p.PB2;

    let interr = ExtiInput::new(Input::new(p.PB4, Pull::Down), p.EXTI4);

    info!("set up i2c");
    let mut i2c = i2c::I2c::new(
        p.I2C1,
        scl,
        sda,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        hz(100000),
        Default::default(),
    );

    let mut sens = MLX90393::new(address, interr, i2c);
    //Timer::after_millis(10).await;
    sens.reset().await;
    //Timer::after_millis(10).await;
    sens.set_sm().await;
    //Timer::after_millis(10).await;
    //sens.read_register(mlx90393::CustomerMemoryArea::WOzThreshold)
    //   .await;
    loop {
        sens.wait_for_measurement().await;
        Timer::after_millis(100).await;
    }

    //loop {
    //   info!("high");
    //   red.set_high();
    //  Timer::after_millis(100).await;
    //
    //       info!("low");
    //      red.set_low();
    //     Timer::after_millis(100).await;
    //}
}
