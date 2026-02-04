#![no_std]
#![no_main]
mod mlx90393;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_futures::join::join_array;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, RawMutex, ThreadModeRawMutex},
    mutex::Mutex,
    signal::Signal,
    watch::Watch,
};
use embedded_io::Write;
use futures::join;
use heapless::{self};

use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, i2c,
    peripherals::{self},
};
use embassy_stm32::{
    rcc::{AHB5Prescaler, AHBPrescaler, APBPrescaler, Sysclk, VoltageScale},
    time::khz,
};

use embassy_stm32::rcc::{PllDiv, PllMul, PllPreDiv, PllSource};
use embassy_time::{Duration, Timer};

use embassy_stm32::usart;
use embedded_hal_async::i2c::I2c;
use mlx90393::sensorgroup::SensorBuilder;

use crate::mlx90393::sensor;
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

async fn init_sensor(
    sensor_param: SensorParams,
    i2c_bus: &'static Mutex<
        CriticalSectionRawMutex,
        i2c::I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
    >,
) -> mlx90393::sensorgroup::Sensor<
    I2cDevice<
        'static,
        CriticalSectionRawMutex,
        i2c::I2c<'static, embassy_stm32::mode::Async, i2c::Master>,
    >,
    Option<embassy_stm32::exti::ExtiInput<'static>>,
> {
    let sensor_address = sensor_param.address;
    let sensor_position = sensor_param.position;
    let sensor_builder = SensorBuilder::new_stm(sensor_address, sensor_position);
    let i2c_device = I2cDevice::new(i2c_bus);
    let mut sensor = sensor_builder.with_i2c(i2c_device).await;
    sensor.set_burst_mode().await;
    sensor
}

#[embassy_executor::task(pool_size = 16)]
async fn send_sensor2(
    mut sensor: mlx90393::sensorgroup::Sensor<
        I2cDevice<
            'static,
            CriticalSectionRawMutex,
            i2c::I2c<'static, embassy_stm32::mode::Async, i2c::Master>,
        >,
        Option<embassy_stm32::exti::ExtiInput<'static>>,
    >,
    uart_bus: &'static Mutex<
        CriticalSectionRawMutex,
        usart::Uart<'static, embassy_stm32::mode::Async>,
    >,
    mut rv: embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, (), 16>,
) {
    let _ = rv.changed().await;
    //let [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15] =
    let mut uart = {
        let uart = WritableDevice::new(uart_bus);
        uart
    };

    loop {
        let val = sensor.send_message(&mut uart).await;
        //let val = sensor.get_message().await;

        match val {
            Ok(v) => {
                debug!("{:#02x}: {:#?}", v.address, v.field);
            }
            Err(v) => {
                //debug!("{:#?}", v);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    static I2C_BUS: StaticCell<
        Mutex<
            CriticalSectionRawMutex,
            i2c::I2c<'_, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
        >,
    > = StaticCell::new();
    static UART_BUS: StaticCell<
        Mutex<CriticalSectionRawMutex, usart::Uart<embassy_stm32::mode::Async>>,
    > = StaticCell::new();

    static UART_INTERFACE: StaticCell<usart::Uart<'_, embassy_stm32::mode::Async>> =
        StaticCell::new();
    static READY: Watch<CriticalSectionRawMutex, (), 16> = Watch::new();
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
    i2c_config.frequency = khz(100);
    i2c_config.sda_pullup = true;
    i2c_config.scl_pullup = true;
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
        //  SensorParams {
        //     address: 0x20,
        //    position: (-6.75, 2.75, 0.0),
        //},
        //SensorParams {
        //address: 0x21,
        //position: (-6.75, 6.75, 0.0),
        //},
    ];
    let sensors = sensor_params.map(|sensor_param| init_sensor(sensor_param, i2c_bus));
    let mut s = [const { None }; 16];

    let len = s.len();
    let mut i = 0;
    for p in sensors {
        //Timer::after_micros(100).await;
        if i < len {
            s[i] = Some(p.await);
        };
        i = i + 1;
    }
    let mut uart = {
        let uart = WritableDevice::new(uart_bus);
        uart
    };

    loop {
        for p in &mut s {
            //let val = sensor.send_message(&mut uart).await;
            //Timer::after_micros(100).await;
            match p {
                Some(sens) => {
                    let val = sens.send_message(&mut uart).await;

                    match val {
                        Ok(v) => {
                            debug!("{:#02x}: {:#?}", v.address, v.field);
                        }
                        Err(v) => {
                            debug!("{:#?}", v);
                        }
                    }
                }
                None => {
                    debug!("Error or missing sensor");
                }
            }
            if let Some(sens) = p {}
        }
    }

    //READY.sender().send(());
}
