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
use futures::{future, join};
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
    peripherals::{
        self, GPDMA1, GPDMA1_CH0, GPDMA1_CH1, GPDMA1_CH2, GPDMA1_CH3, I2C1, PA8, PB12, USART1,
    },
    time::hz,
};
use embassy_stm32::{
    rcc::{mux, AHB5Prescaler, AHBPrescaler, APBPrescaler, Sysclk, VoltageScale},
    time::khz,
};

use embassy_stm32::rcc::{PllDiv, PllMul, PllPreDiv, PllSource};
use embassy_time::{Duration, Timer};
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

struct SensorParams {
    address: u8,
    position: (f32, f32, f32),
}

#[embassy_executor::task(pool_size = 16)]
async fn send_sensor2(
    sensor_params: [SensorParams; 16],
    uart_bus: &'static Mutex<NoopRawMutex, usart::Uart<'static, embassy_stm32::mode::Async>>,
    i2c_bus: &'static Mutex<
        NoopRawMutex,
        i2c::I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
    >,
) {
    let [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15] =
        sensor_params.map(|sensor_param| async move {
            let sensor_address = sensor_param.address;
            let sensor_position = sensor_param.position;
            let sensor_builder = SensorBuilder::new_stm(sensor_address, sensor_position);
            let i2c_device = I2cDevice::new(&i2c_bus);
            let mut sensor = sensor_builder.with_i2c(i2c_device).await;
            let mut uart = WritableDevice::new(&uart_bus);
            return (sensor, uart);
        });
    let (
        mut s0,
        mut s1,
        mut s2,
        mut s3,
        mut s4,
        mut s5,
        mut s6,
        mut s7,
        mut s8,
        mut s9,
        mut s10,
        mut s11,
        mut s12,
        mut s13,
        mut s14,
        mut s15,
    ) = join!(s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15);

    loop {
        //let val = sensor.send_message(&mut uart).await;

        let v0 = s0.0.get_message();
        let v1 = s1.0.get_message();
        let v2 = s2.0.get_message();
        let v3 = s3.0.get_message();
        let v4 = s4.0.get_message();
        let v5 = s5.0.get_message();
        let v6 = s6.0.get_message();
        let v7 = s7.0.get_message();
        let v8 = s8.0.get_message();
        let v9 = s9.0.get_message();
        let v10 = s10.0.get_message();
        let v11 = s11.0.get_message();
        let v12 = s12.0.get_message();
        let v13 = s13.0.get_message();
        let v14 = s14.0.get_message();
        let v15 = s15.0.get_message();
        let values = join!(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15);
        for val in [
            values.0, values.1, values.2, values.3, values.4, values.5, values.6, values.7,
            values.8, values.9, values.10, values.11, values.12, values.13, values.14, values.15,
        ] {
            if let Ok(v) = val {
                debug!("{:#?}: {:#?}", v.address, v.field);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    static I2C_BUS: StaticCell<
        Mutex<NoopRawMutex, i2c::I2c<'_, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>>,
    > = StaticCell::new();
    static UART_BUS: StaticCell<Mutex<NoopRawMutex, usart::Uart<embassy_stm32::mode::Async>>> =
        StaticCell::new();

    static UART_INTERFACE: StaticCell<usart::Uart<'_, embassy_stm32::mode::Async>> =
        StaticCell::new();
    let mut config = embassy_stm32::Config::default();

    // Fine-tune PLL1 dividers/multipliers

    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,

        prediv: PllPreDiv::DIV1, // PLLM = 1 → HSI / 1 = 16 MHz

        mul: PllMul::MUL30, // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO

        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)

        // divq: Some(PllDiv::DIV10), // PLLQ = 10 → 48 MHz (NOT USED)
        divq: None,

        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (USBOTG)

        frac: Some(0), // Fractional part (enabled)
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;

    config.rcc.apb1_pre = APBPrescaler::DIV1;

    config.rcc.apb2_pre = APBPrescaler::DIV1;

    config.rcc.apb7_pre = APBPrescaler::DIV1;

    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;

    // voltage scale for max performance

    config.rcc.voltage_scale = VoltageScale::RANGE1;

    // route PLL1_P into the USB‐OTG‐HS block

    config.rcc.sys = Sysclk::PLL1_R;
    let p = embassy_stm32::init(config);

    info!("Hello World!");
    let UART_RX = p.PA8;
    let UART_TX = p.PB12;

    let uart_interface = usart::Uart::new(
        p.USART1,
        UART_RX,
        UART_TX,
        Irqs,
        p.GPDMA1_CH2,
        p.GPDMA1_CH3,
        usart::Config::default(),
    )
    .unwrap();
    let uart_bus_mutex = Mutex::new(uart_interface);
    let uart_bus = UART_BUS.init(uart_bus_mutex);
    let sda = p.PB1;
    let scl = p.PB2;

    let mut i2c_config = i2c::Config::default();
    i2c_config.timeout = Duration::from_millis(500);
    i2c_config.frequency = khz(400);
    let i2cport = i2c::I2c::new(
        p.I2C1,
        scl,
        sda,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        i2c_config,
    );

    Timer::after_millis(100).await;
    let i2c_bus = Mutex::new(i2cport);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    let sensor_params = [
        SensorParams {
            address: 0x0C,
            position: (6.75, -6.75, 0.0),
        },
        SensorParams {
            address: 0x0D,
            position: (6.75, -2.75, 0.0),
        },
        SensorParams {
            address: 0x0E,
            position: (6.75, 2.75, 0.0),
        },
        SensorParams {
            address: 0x0F,
            position: (6.75, 6.75, 0.0),
        },
        SensorParams {
            address: 0x10,
            position: (2.75, -6.75, 0.0),
        },
        SensorParams {
            address: 0x11,
            position: (2.75, -2.75, 0.0),
        },
        SensorParams {
            address: 0x12,
            position: (2.75, 2.75, 0.0),
        },
        SensorParams {
            address: 0x13,
            position: (2.75, 6.75, 0.0),
        },
        SensorParams {
            address: 0x14,
            position: (-2.75, -6.75, 0.0),
        },
        SensorParams {
            address: 0x15,
            position: (-2.75, -2.75, 0.0),
        },
        SensorParams {
            address: 0x16,
            position: (-2.75, 2.75, 0.0),
        },
        SensorParams {
            address: 0x17,
            position: (-2.75, 6.75, 0.0),
        },
        SensorParams {
            address: 0x18,
            position: (-6.75, -6.75, 0.0),
        },
        SensorParams {
            address: 0x19,
            position: (-6.75, -2.75, 0.0),
        },
        SensorParams {
            address: 0x20,
            position: (-6.75, 2.75, 0.0),
        },
        SensorParams {
            address: 0x21,
            position: (-6.75, 6.75, 0.0),
        },
    ];

    let _ = spawner.spawn(send_sensor2(sensor_params, uart_bus, i2c_bus));
}
