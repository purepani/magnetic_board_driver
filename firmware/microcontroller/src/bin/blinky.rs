#![no_std]
#![no_main]
mod mlx90393;

use data_transfer::conversions::MagneticField;
use embedded_io::Write;
use heapless::{self, String};
use postcard;

use defmt::{debug, info, Formatter};
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

use embassy_stm32::usart;
use embedded_hal_async::i2c::{I2c, Operation};
use embedded_hal_bus::util::AtomicCell;
use mlx90393::sensorgroup::Sensor;

use mlx90393::MLX90393;
//use embedded_hal::blocking::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
        USART1 => usart::InterruptHandler<peripherals::USART1>;

    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //let address_write: u8 = 0b0001110;
    //let address_read: u8 = 0b0001111;
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let sda = p.PB1;
    let scl = p.PB2;

    info!("set up i2c");
    let i2c = i2c::I2c::new(
        p.I2C1,
        scl,
        sda,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        hz(400000),
        Default::default(),
    );

    //let address: u8 = 0x12;
    //let pin = Input::new(p.PA10, Pull::Down);
    //let interr = ExtiInput::new(pin, p.EXTI10);

    //let mut led = Output::new(p.PB4, Level::High, Speed::Low);
    //let mut red = Output::new(p.PB7, Level::High, Speed::Low);

    let uart_rx = p.PA8;
    let uart_tx = p.PB12;

    let mut uart_interface = usart::Uart::new(
        p.USART1,
        uart_rx,
        uart_tx,
        Irqs,
        p.GPDMA1_CH2,
        p.GPDMA1_CH3,
        usart::Config::default(),
    )
    .unwrap();

    //let i2c1 = AtomicCell::new(i2c);
    //let i2c2 = AtomicCell::new(i2c);

    //let mut sensor = Sensor::new_stm(0x0C, (6.75, -6.75, 0.0), p.PB0, p.EXTI0, i2c).await;
    let mut sensor = Sensor::new_stm(0x0D, (6.75, -2.25, 0.0), p.PB14, p.EXTI14, i2c).await;
    //    (0x0E, (6.75, 2.25, 0.0), p.PB13, p.EXTI13);
    //   (0x0F, (6.75, 6.75, 0.0), p.PA10, p.EXTI10);
    //(0x10, (2.25, -6.75, 0.0), p.PB4, p.EXTI4);
    //(0x11, (2.25, -2.25, 0.0), p.PB3, p.EXTI10);
    //(0x12, (2.25, 2.25, 0.0), p.PA10, p.EXTI10);
    //(0x13, (2.25, 6.75, 0.0), p.PA10, p.EXTI10);
    //(0x14, (-2.25, -6.75, 0.0), p.PA12, p.EXTI12);
    //(0x15, (-2.25, -2.25, 0.0), p.PB5, p.EXTI5);
    //(0x16, (-2.25, 2.25, 0.0), p.PA10, p.EXTI10);
    //(0x17, (-2.25, 6.75, 0.0), p.PA10, p.EXTI10);
    //(0x18, (-6.75, -6.75, 0.0), p.PA9, p.EXTI9);
    //(0x19, (-6.75, -2.25, 0.0), p.PA2, p.EXTI2);
    //(0x1A, (-6.75, 2.25, 0.0), p.PA10, p.EXTI10);
    //(0x1B, (-6.75, 6.75, 0.0), p.PA10, p.EXTI10);

    //let mut sensor = Sensor::new_stm(0x12, (0.0, 0.0, 0.0), p.PA10, p.EXTI10, i2c).await;

    loop {
        let val = sensor.send_message(&mut uart_interface).await;
        debug!("{:#?}", val);
    }

    //let mut sens = MLX90393::new(address, interr, i2c);
    //Timer::after_millis(100).await;
    //sens.reset().await;
    //Timer::after_millis(100).await;
    //sens.set_measurement_configuration().await;
    //match sens.state {
    //None => info!("No sensor state able to be read."),
    //Some(_) => info!("Set Sensor State"),
    //}
    //Timer::after_millis(200).await;
    //sens.set_burst::<true, true, true, true>().await;
    //Timer::after_millis(200).await;

    //sens.read_register(mlx90393::CustomerMemoryArea::WOzThreshold)
    //   .await;
    //loop {
    //let mut buffer = [0; 1];
    //let _command = uart_interface.blocking_read(&mut buffer);
    //sens.set_single_measurmenet::<true, true, true, true>()
    //.await;
    //let (status, field) = sens.get_field::<true, true, true, true>().await;

    //match field {
    //Some(x) => info!("{:#?}", x),
    //None => info!("No field found"),
    //}
    //let message = field.map(|f| data_transfer::messaging::Message::new(f));

    //if let Some(msg) = message {
    //let _ = postcard::to_eio(&b, &mut uart_interface);
    // let res = msg.write_to(&mut uart_interface);
    //if let Err(err) = res {
    //info!("{:#?}", err)
    //}
    //}
    //Timer::after_millis(100).await;

    //usart.blocking_write(&buffer);
    //info!("Sent uart!");
    //match field {
    //   Some(x) => info!("{:?}", x),
    //  None => info!("No field measured"),
    //};
    //}

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
