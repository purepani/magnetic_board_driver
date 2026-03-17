#![no_std]
#![no_main]

pub mod app;
pub mod handlers;
pub mod mlx90393;


use static_cell::StaticCell;
use app::{Context, MyApp, STORAGE, AppServer};
use postcard_rpc::server::{Server, Dispatch};


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
use mlx90393::sensorgroup::{SensorGroupBuilder, SensorBuilder};

use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
        USART1 => usart::InterruptHandler<peripherals::USART1>;

    }
);



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    static I2C_BUS: StaticCell<
        Mutex<
            CriticalSectionRawMutex,
            i2c::I2c<'_, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
        >,
    > = StaticCell::new();
    static UART_TX_BUS: StaticCell<
        Mutex<CriticalSectionRawMutex, usart::Uart<embassy_stm32::mode::Async>>,
    > = StaticCell::new();
    static UART_RX_BUS: StaticCell<
        Mutex<CriticalSectionRawMutex, usart::RingBufferedUartRx>,
    > = StaticCell::new();
    static UART_RX_BUFFER: StaticCell<Mutex<CriticalSectionRawMutex, [u8; 256]>> = StaticCell::new();
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
    let (uart_tx, uart_rx) = uart_interface.split();
    let mut rx_buffer = [0u8; 256];
    let rx_buffer = UART_RX_BUFFER.init(Mutex::new(rx_buffer));
    let uart_rx = uart_rx.into_ring_buffered(rx_buffer.get_mut());
    
    //let uart_rx_bus_mutex = Mutex::new(uart_rx);
    //let uart_rx_bus = UART_RX_BUS.init(uart_rx_bus_mutex);

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
    let i2c_bus = Mutex::new(i2cport);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    let i2c_devices: [_; 16] = core::array::from_fn(|_| I2cDevice::new(i2c_bus));

    let sensor_builders: [_; 16] = core::array::from_fn(|i| SensorBuilder::new_stm(0x0C+(i as u8), (0.0, 0.0, 0.0)));
    let mut sensor_group_builder = SensorGroupBuilder::new_stm(0, sensor_builders);
    let mut sensor_groups = [sensor_group_builder.with_i2c(i2c_devices).await; 1];


    let impls = STORAGE.init(uart_rx, uart_tx);
    match impls {
        Some(_) => {},
        None => {
            debug!("Storage Failed");
            return
        }
    }
    let (rx_impl, tx_impl) = impls.unwrap();

    static PACKET_RX_BUF: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0u8; 256]);

    let context = Context {
        sensor_groups
    };
    let dispatcher = MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: AppServer =
        Server::new(tx_impl, rx_impl, PACKET_RX_BUF.take(), dispatcher, vkk);
    loop {
        let _ = server.run().await;

    }
    // loop {
    //     for i in 0..sensor_group.num_sensors() {
    //         let message = sensor_group.get_message(i).await;
    //         debug!("{:?}", message)
            
    //     }
    // }
    
}
