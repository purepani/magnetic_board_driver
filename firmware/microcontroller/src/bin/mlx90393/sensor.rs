#![no_std]
use super::commands::{Command, CommandData, RunCommand, RM};
use crate::mlx90393::commands::MagneticFieldReturnFlags;
use crate::mlx90393::states::Burst;
use crate::mlx90393::states::Idle;
use crate::mlx90393::states::Measured;
use crate::mlx90393::states::Measuring;
use crate::mlx90393::states::NoMode;
use crate::mlx90393::states::SingleMeasurement;
use crate::mlx90393::states::WakeOnChange;
use data_transfer::conversions::MagneticBits;
use data_transfer::memory::{Register, TempRef};

use super::states::SensorState;
use bitflags::bitflags;
use data_transfer::conversions::MagneticField;
use data_transfer::memory::{Gain, HallConf, Res3D, TempOffset, TemperatureCompensation};
//use bitvec::prelude::*;
use defmt::{debug, info, Format};
//use embassy_stm32::i2c::Error;
use embassy_time::Timer;
//use embedded_hal::digital::v2::InputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::I2c;
//use heapless::Vec;

bitflags! {
    struct StatusFlags: u8 {
        const burst = 0b10000000;
        const woc = 0b01000000;
        const sm = 0b00100000;
        const error = 0b00010000;
        const sed = 0b00001000;
        const rs = 0b00000100;
        const data = 0b00000011;
    }
}

#[derive(defmt::Format)]
pub struct Status {
    pub burst_mode: bool,
    pub woc_mode: bool,
    pub sm_mode: bool,
    pub error: bool,
    pub sed: bool,
    pub rs: bool,
    pub data: u8,
}

impl Status {
    fn from_u8(status: &u8) -> Self {
        let x = StatusFlags::from_bits_retain(*status);
        Status {
            burst_mode: x.contains(StatusFlags::burst),
            woc_mode: x.contains(StatusFlags::woc),
            sm_mode: x.contains(StatusFlags::sm),
            error: x.contains(StatusFlags::error),
            sed: x.contains(StatusFlags::sed),
            rs: x.contains(StatusFlags::rs),
            data: (x & StatusFlags::data).bits(),
        }
    }
}

pub struct MLX90393<I, P> {
    pub address: u8,
    pub interrupt: P,
    i2c: I,

    pub state: Option<MLXSettings>,
}

#[derive(Clone, Copy, Format)]
pub struct MLXSettings {
    resolution: Res3D,
    gain: Gain,
    temperature_compensation: TemperatureCompensation,
    hall_configuration: HallConf,
    temp_ref: TempRef,
}

impl<I: I2c, P: Wait> MLX90393<I, P> {
    pub fn new(address: u8, interrupt: P, i2c: I) -> Self {
        Self {
            address,
            interrupt,
            i2c,
            state: None,
        }
    }

    pub async fn run_command<C, T, const M: usize, const N: usize>(
        &mut self,
        command: C,
    ) -> (Status, [u8; N])
    where
        C: RunCommand<T, M, N>,
    {
        let commands = command.write_command();
        let mut buffer = command.read_buffer();
        let _ = self
            .i2c
            .write_read(self.address, &commands, &mut buffer)
            .await;
        let status = Status::from_u8(&buffer[0]);

        (status, buffer)
    }

    pub async fn run_command_with_wait<C, T, const M: usize, const N: usize>(
        &mut self,
        command: C,
        millis: u64,
    ) -> (Status, [u8; N])
    where
        C: RunCommand<T, M, N>,
    {
        let commands = command.write_command();
        let mut buffer = command.read_buffer();
        let _ = self.i2c.write(self.address, &commands).await;
        Timer::after_millis(millis).await;

        let _ = self.i2c.read(self.address, &mut buffer).await;
        let status = Status::from_u8(&buffer[0]);
        debug!("Status: {:#?}", &status);

        (status, buffer)
    }

    pub async fn reset(&mut self) {
        let exit = Command::exit();
        let (_status, _) = self.run_command(exit).await;
        Timer::after_micros(1000).await;
        let reset = Command::reset();
        let (_status, _) = self.run_command(reset).await;
        Timer::after_micros(1500).await;
    }

    pub async fn set_sm<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(&mut self) {
        info!("Settings Mode to Single Measurement.");
        let _ = self
            .run_command(Command::single_measurement::<X, Y, Z, TEMP>())
            .await;
    }

    pub async fn read_register<const R: u8>(&mut self) -> Register<R> {
        let command = Command::read_register(R);
        let (status, data) = self.run_command_with_wait(command, 100).await;
        let [_, data1, data2] = data;
        let d = [data1, data2];
        Register::<R>::new(d)
    }

    pub async fn set_measurement_configuration(&mut self) -> &mut Self {
        self.state = self.get_measurement_configuration().await;
        debug!("State: {}", self.state);
        self
    }

    pub async fn get_measurement_configuration(&mut self) -> Option<MLXSettings> {
        Timer::after_millis(150).await;
        let data_bits = &self.read_register::<0x00>().await;
        let gain = data_bits.gain();
        let hall_configuration = data_bits.hall_conf()?;
        Timer::after_millis(150).await;

        let data_bits = &self.read_register::<0x02>().await;
        let resolution = data_bits.resolution();
        Timer::after_millis(150).await;

        let data_bits = &self.read_register::<0x01>().await;
        let temperature_compensation = data_bits.temperature_compensation();
        Timer::after_millis(150).await;

        let data_bits = &self.read_register::<0x24>().await;
        let temp_ref = data_bits.temperature_reference();

        Some(MLXSettings {
            resolution,
            gain,
            hall_configuration,
            temperature_compensation,
            temp_ref,
        })
    }

    pub async fn set_woc<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(&mut self) {
        info!("Settings Mode to Wake On Change.");
        let _ = self
            .run_command(Command::start_wake_on_change::<X, Y, Z, TEMP>())
            .await;
    }
    pub async fn set_burst<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(
        &mut self,
    ) {
        info!("Settings Mode to Burst.");
        let (status, buffer) = self
            .run_command(Command::start_burst::<X, Y, Z, TEMP>())
            .await;
        info!("{:#?}", status);
    }
    pub async fn set_single_measurmenet<
        const X: bool,
        const Y: bool,
        const Z: bool,
        const TEMP: bool,
    >(
        &mut self,
    ) {
        info!("Settings Mode to Single Measurement.");
        self.run_command(Command::single_measurement::<X, Y, Z, TEMP>())
            .await;
    }

    pub async fn get_measurement<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(
        &mut self,
    ) -> (Status, MagneticBits) {
        //info!("Waiting for interrupt.");
        let _ = self.interrupt.wait_for_high().await;
        //info!("Received Interrupt");
        let (status, mbits) = {
            match (X, Y, Z, TEMP) {
                (true, true, true, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, true, true, true>())
                        .await;
                    let [_, t1, t2, x1, x2, y1, y2, z1, z2] = buffer;
                    let x = Some([x1, x2]);
                    let y = Some([y1, y2]);
                    let z = Some([z1, z2]);
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, true, true, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, true, true, false>())
                        .await;
                    let [_, x1, x2, y1, y2, z1, z2] = buffer;
                    let x = Some([x1, x2]);
                    let y = Some([y1, y2]);
                    let z = Some([z1, z2]);
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, true, false, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, true, false, true>())
                        .await;
                    let [_, t1, t2, x1, x2, y1, y2] = buffer;
                    let x = Some([x1, x2]);
                    let y = Some([y1, y2]);
                    let z = None;
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, true, false, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, true, false, false>())
                        .await;
                    let [_, x1, x2, y1, y2] = buffer;
                    let x = Some([x1, x2]);
                    let y = Some([y1, y2]);
                    let z = None;
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, false, true, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, false, true, true>())
                        .await;
                    let [_, t1, t2, x1, x2, z1, z2] = buffer;
                    let x = Some([x1, x2]);
                    let y = None;
                    let z = Some([z1, z2]);
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, false, true, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, false, true, false>())
                        .await;
                    let [_, x1, x2, z1, z2] = buffer;
                    let x = Some([x1, x2]);
                    let y = None;
                    let z = Some([z1, z2]);
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, false, false, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, false, false, true>())
                        .await;
                    let [_, t1, t2, x1, x2] = buffer;
                    let x = Some([x1, x2]);
                    let y = None;
                    let z = None;
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (true, false, false, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<true, false, false, false>())
                        .await;
                    let [_, x1, x2] = buffer;
                    let x = Some([x1, x2]);
                    let y = None;
                    let z = None;
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, true, true, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, true, true, true>())
                        .await;
                    let [_, t1, t2, y1, y2, z1, z2] = buffer;
                    let x = None;
                    let y = Some([y1, y2]);
                    let z = Some([z1, z2]);
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, true, true, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, true, true, true>())
                        .await;
                    let [_, t1, t2, y1, y2, z1, z2] = buffer;
                    let x = None;
                    let y = Some([y1, y2]);
                    let z = Some([z1, z2]);
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, true, false, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, true, false, true>())
                        .await;
                    let [_, t1, t2, y1, y2] = buffer;
                    let x = None;
                    let y = Some([y1, y2]);
                    let z = None;
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, true, false, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, true, false, false>())
                        .await;
                    let [_, y1, y2] = buffer;
                    let x = None;
                    let y = Some([y1, y2]);
                    let z = None;
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, false, true, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, false, true, true>())
                        .await;
                    let [_, t1, t2, z1, z2] = buffer;
                    let x = None;
                    let y = None;
                    let z = Some([z1, z2]);
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, false, true, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, false, true, false>())
                        .await;
                    let [_, z1, z2] = buffer;
                    let x = None;
                    let y = None;
                    let z = Some([z1, z2]);
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, false, false, true) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, false, false, true>())
                        .await;
                    let [_, t1, t2] = buffer;
                    let x = None;
                    let y = None;
                    let z = None;
                    let temp = Some([t1, t2]);
                    (status, MagneticBits::new(x, y, z, temp))
                }
                (false, false, false, false) => {
                    let (status, buffer) = self
                        .run_command(Command::read_measurement::<false, false, false, false>())
                        .await;
                    let [_] = buffer;
                    let x = None;
                    let y = None;
                    let z = None;
                    let temp = None;
                    (status, MagneticBits::new(x, y, z, temp))
                }
            }
        };

        //info!("{}", status);
        (status, mbits)
    }

    pub async fn get_field<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(
        &mut self,
    ) -> (Status, Option<MagneticField>) {
        let state = self.state;
        let (status, mbits) = self.get_measurement::<X, Y, Z, TEMP>().await;
        //info!("{:#?}", status);

        (
            status,
            state.and_then(|state| {
                MagneticField::from_mbits(
                    mbits,
                    state.temp_ref,
                    state.temperature_compensation,
                    state.gain,
                    state.resolution,
                    state.hall_configuration,
                )
            }),
        )
    }

    pub async fn has_measured(&mut self) {
        let _ = self.interrupt.wait_for_high().await;
    }
}

pub struct Sensor<S, T, I, P> {
    state: SensorState<S, T>,
    internal: MLX90393<I, P>,
}

impl<S, T, I: I2c, P: Wait> Sensor<S, T, I, P> {
    pub async fn reset(mut self) -> Sensor<Idle, NoMode, I, P> {
        self.internal.reset().await;
        Sensor {
            state: SensorState {
                state: Idle,
                mode: NoMode,
            },
            internal: self.internal,
        }
    }
}

impl<I: I2c, P: Wait> Sensor<Idle, NoMode, I, P> {
    pub async fn new(address: u8, interrupt: P, i2c: I) -> Sensor<Idle, NoMode, I, P> {
        let sensor = Sensor {
            state: SensorState {
                state: Idle,
                mode: NoMode,
            },
            internal: MLX90393::new(address, interrupt, i2c),
        };
        sensor.reset().await
    }
}

impl<I: I2c, P: Wait> Sensor<Idle, NoMode, I, P> {
    pub async fn single_measurement<
        const X: bool,
        const Y: bool,
        const Z: bool,
        const TEMP: bool,
    >(
        mut self,
    ) -> Sensor<Measuring, SingleMeasurement, I, P> {
        self.internal
            .set_single_measurmenet::<X, Y, Z, TEMP>()
            .await;
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }

    pub async fn burst<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(
        mut self,
    ) -> Sensor<Measuring, Burst, I, P> {
        self.internal.set_burst::<X, Y, Z, TEMP>().await;
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }

    pub async fn wake_on_change<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(
        mut self,
    ) -> Sensor<Measuring, WakeOnChange, I, P> {
        self.internal.set_woc::<X, Y, Z, TEMP>().await;
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }
}

impl<T, I: I2c, P: Wait> Sensor<Measuring, T, I, P> {
    pub async fn has_measured(mut self) -> Sensor<Measured, T, I, P> {
        self.internal.has_measured().await;
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }
}

impl<I: I2c, P: Wait> Sensor<Measured, Burst, I, P> {
    pub async fn has_measured(self) -> Sensor<Measuring, Burst, I, P> {
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }
}

impl<I: I2c, P: Wait> Sensor<Measured, WakeOnChange, I, P> {
    pub async fn has_measured(self) -> Sensor<Measuring, WakeOnChange, I, P> {
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }
}

impl<I: I2c, P: Wait> Sensor<Measured, SingleMeasurement, I, P> {
    pub async fn has_measured(self) -> Sensor<Idle, NoMode, I, P> {
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }
}

impl<T, I: I2c, P: Wait> Sensor<Measured, T, I, P> {
    pub async fn exit(self) -> Sensor<Idle, NoMode, I, P> {
        Sensor {
            state: self.state.into(),
            internal: self.internal,
        }
    }
}
