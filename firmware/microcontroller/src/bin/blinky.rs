#![no_std]
#![no_main]
mod mlx90393;

use core::cell::RefCell;

use data_transfer::conversions::MagneticField;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, RawMutex},
    mutex::Mutex,
};
use embedded_io::Write;
use heapless::{self, String, Vec};
use postcard;

use defmt::{debug, info, Formatter};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::{self, AnyChannel, ExtiInput},
    flash::Async,
    gpio::{self, AnyPin, Input, Level, Output, Pin, Pull, Speed},
    i2c, interrupt,
    peripherals::{self, GPDMA1, GPDMA1_CH0, GPDMA1_CH1, GPDMA1_CH2, GPDMA1_CH3, I2C1, USART1},
    time::hz,
};
use embassy_time::Timer;
use embedded_hal_async::digital::Wait;

use embassy_stm32::usart;
use embedded_hal_async::i2c::{I2c, Operation};
use embedded_hal_bus::{i2c::RefCellDevice, util::AtomicCell};
use mlx90393::sensorgroup::{Sensor, SensorBuilder};

use mlx90393::MLX90393;
use static_cell::StaticCell;
//use embedded_hal::blocking::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
        USART1 => usart::InterruptHandler<peripherals::USART1>;

    }
);

struct WritableDevice<'a, M: RawMutex, BUS> {
    bus: &'a Mutex<M, BUS>,
}

impl<'a, M: RawMutex, BUS> WritableDevice<'a, M, BUS> {
    pub fn new(bus: &'a Mutex<M, BUS>) -> Self {
        Self { bus }
    }
}

impl<'a, M: RawMutex, BUS> embedded_io_async::ErrorType for WritableDevice<'a, M, BUS>
where
    BUS: embedded_io_async::ErrorType,
{
    type Error = BUS::Error;
}

impl<M, BUS> embedded_io_async::Write for WritableDevice<'_, M, BUS>
where
    M: RawMutex + 'static,
    BUS: embedded_io_async::Write + 'static,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut bus = self.bus.lock().await;
        bus.write(buf).await
    }
}

#[embassy_executor::task]
async fn send_sensor() {
    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, i2c::I2c<'_, I2C1, GPDMA1_CH0, GPDMA1_CH1>>> =
        StaticCell::new();
    static UART_BUS: StaticCell<Mutex<NoopRawMutex, usart::Uart<USART1, GPDMA1_CH2, GPDMA1_CH3>>> =
        StaticCell::new();
    let p = embassy_stm32::init(Default::default());
    let uart_rx = p.PA8;
    let uart_tx = p.PB12;
    let uart_interface = usart::Uart::new(
        p.USART1,
        uart_rx,
        uart_tx,
        Irqs,
        p.GPDMA1_CH2,
        p.GPDMA1_CH3,
        usart::Config::default(),
    )
    .unwrap();
    let uart_bus = Mutex::new(uart_interface);
    let uart_bus = UART_BUS.init(uart_bus);
    let sda = p.PB1;
    let scl = p.PB2;

    let i2cport = i2c::I2c::new(
        p.I2C1,
        scl,
        sda,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        hz(400000),
        Default::default(),
    );

    let i2c_bus = Mutex::new(i2cport);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    let mut uart1 = WritableDevice::new(&uart_bus);
    let mut uart2 = WritableDevice::new(&uart_bus);
    //let mut uart3 = WritableDevice::new(&uart_bus);
    //let mut uart4 = WritableDevice::new(&uart_bus);
    //let mut uart5 = WritableDevice::new(&uart_bus);
    //let mut uart6 = WritableDevice::new(&uart_bus);
    //let mut uart7 = WritableDevice::new(&uart_bus);
    //let mut uart8 = WritableDevice::new(&uart_bus);
    //let mut uart9 = WritableDevice::new(&uart_bus);
    //let mut uart10 = WritableDevice::new(&uart_bus);
    //let mut uart11 = WritableDevice::new(&uart_bus);
    //let mut uart12 = WritableDevice::new(&uart_bus);
    //let mut uart13 = WritableDevice::new(&uart_bus);
    //let mut uart14 = WritableDevice::new(&uart_bus);
    //let mut uart15 = WritableDevice::new(&uart_bus);
    //let mut uart16 = WritableDevice::new(&uart_bus);

    let sensors = (
        SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x0C, (6.75, -6.75, 0.0)),
        SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x0D, (6.75, -2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x0E, (6.75, 2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x0F, (6.75, 6.75, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x10, (2.25, -6.75, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x11, (2.25, -2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x12, (2.25, 2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x13, (2.25, 6.75, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x14, (-2.75, -6.75, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x15, (-2.75, -2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x16, (-2.75, 2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x17, (-2.75, 6.75, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x18, (-6.75, -6.75, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x19, (-6.75, -2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x1A, (-6.75, 2.25, 0.0)),
        //SensorBuilder::<gpio::AnyPin, exti::AnyChannel>::new_stm(0x1B, (-6.75, 6.75, 0.0)),
    );
    let mut sensors = (
        sensors.0.with_i2c(I2cDevice::new(&i2c_bus)).await,
        sensors.1.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.2.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.3.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.4.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.5.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.6.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.7.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.8.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.9.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.10.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.11.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.12.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.13.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.14.with_i2c(I2cDevice::new(&i2c_bus)).await,
        //sensors.15.with_i2c(I2cDevice::new(&i2c_bus)).await,
    );

    //let mut sensor = sensor_builder.with_i2c(i2cport).await;
    loop {
        let val1 = sensors.0.send_message(&mut uart1);
        let val2 = sensors.1.send_message(&mut uart2);
        //let val3 = sensors.2.send_message(&mut uart3);
        //let val4 = sensors.3.send_message(&mut uart4);
        //let val5 = sensors.4.send_message(&mut uart5);
        //let val6 = sensors.5.send_message(&mut uart6);
        //let val7 = sensors.6.send_message(&mut uart7);
        //let val8 = sensors.7.send_message(&mut uart8);
        //let val9 = sensors.8.send_message(&mut uart9);
        //let val10 = sensors.9.send_message(&mut uart10);
        //let val11 = sensors.10.send_message(&mut uart11);
        //let val12 = sensors.11.send_message(&mut uart12);
        //let val13 = sensors.12.send_message(&mut uart13);
        //let val14 = sensors.13.send_message(&mut uart14);
        //let val15 = sensors.14.send_message(&mut uart15);
        //let val16 = sensors.15.send_message(&mut uart16);
        let (
            val1,
            val2,
            //val3,
            //val4,
            //val5,
            //val6,
            //val7,
            //val8,
            //val9,
            //val10,
            //val11,
            //val12,
            //val13,
            //val14,
            //val15,
            //val16,
        ) = futures::join!(
            val1, val2, //val3, val4, val5, val6, val7, val8, val9, val10, val11, val12, val13,
            //val14, val15, val16
        );
        debug!("{:#?}", val1);
        debug!("{:#?}", val2);
        //debug!("{:#?}", val3);
        //debug!("{:#?}", val4);
        //debug!("{:#?}", val5);
        //debug!("{:#?}", val6);
        //debug!("{:#?}", val7);
        //debug!("{:#?}", val8);
        //debug!("{:#?}", val9);
        //debug!("{:#?}", val10);
        //debug!("{:#?}", val11);
        //debug!("{:#?}", val12);
        //debug!("{:#?}", val13);
        //debug!("{:#?}", val14);
        //debug!("{:#?}", val15);
        //debug!("{:#?}", val16);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    //let address_write: u8 = 0b0001110;
    //let address_read: u8 = 0b0001111;
    info!("Hello World!");

    //static I2C_BUS: StaticCell<Mutex<CriticalSectionRawMutex, i2c::I2c<'_, I2C1, GPDMA1_CH0, GPDMA1_CH1>>> =
    ////   StaticCell::new();

    //static UART_BUS: StaticCell<Mutex<CriticalSectionRawMutex, usart::Uart<USART1, GPDMA1_CH2, GPDMA1_CH3>>> =
    //StaticCell::new();

    //info!("set up i2c");
    //let i2c = i2c::I2c::new(
    //p.I2C1,
    //scl,
    //sda,
    //Irqs,
    //p.GPDMA1_CH0,
    //p.GPDMA1_CH1,
    //hz(400000),
    //Default::default(),
    //);
    //let i2c_bus: Mutex<CriticalSectionRawMutex, i2c::I2c<'_, I2C1, GPDMA1_CH0, GPDMA1_CH1>> = Mutex::new(i2c);
    //let i2c_bus = I2C_BUS.init(i2c_bus);

    //let address: u8 = 0x12;
    //let pin = Input::new(p.PA10, Pull::Down);
    //let interr = ExtiInput::new(pin, p.EXTI10);

    //let mut led = Output::new(p.PB4, Level::High, Speed::Low);
    //let mut red = Output::new(p.PB7, Level::High, Speed::Low);

    //let uart_bus = Mutex::new(uart_interface);
    //let uart_bus = UART_BUS.init(uart_bus);
    //let uart1 = WritableDevice::new(&uart_bus);
    //let uart2 = WritableDevice::new(&uart_bus);

    //let i2c1 = I2cDevice::new(&i2c_bus);
    //let i2c2 = I2cDevice::new(&i2c_bus);
    //let i2c3 = I2cDevice::new(&i2c_bus);
    //let i2c4 = I2cDevice::new(&i2c_bus);

    //let sensor1 = SensorBuilder::new_stm_degraded(0x0C, (6.75, -6.75, 0.0), p.PB0, p.EXTI0);
    //let sensor2 = SensorBuilder::new_stm_degraded(0x0D, (6.75, -2.25, 0.0), p.PB14, p.EXTI14);
    let _ = spawner.spawn(send_sensor());
    //let y = spawner.spawn(send_sensor(sensor2, uart_bus, i2c_bus));
    //match y {
    //Ok(_) => {}
    //Err(err) => {
    //info!("Failed sensor2: {}", err)
    //}
    //}
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

    //loop {
    //let val1 = sensor1.send_message(&mut uart_interface);
    //let val2 = sensor2.send_message(&mut uart_interface);
    //futures::join!(val1, val2);
    //debug!("{:#?}", val1);
    //debug!("{:#?}", val2);
    //}

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
