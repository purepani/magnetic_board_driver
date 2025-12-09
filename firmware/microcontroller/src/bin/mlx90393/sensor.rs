#![no_std]

use super::commands::{Command, RunCommand};
use data_transfer::conversions::MagneticBits;
use data_transfer::memory::{Register, TempRef};

use bitflags::bitflags;
use data_transfer::conversions::MagneticField;
use data_transfer::memory::{Gain, HallConf, Res3D, TemperatureCompensation};
//use bitvec::prelude::*;
use defmt::{debug, info, Format};
//use embassy_stm32::i2c::Error;
use embassy_time::Timer;
//use embedded_hal::digital::v2::InputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::I2c;
//use heapless::Vec;

const T_STBY_MICRO: u64 = 264;
const T_ACTIVE_MICRO: u64 = 264;
const T_CONV_END_MICRO: u64 = 264;

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
    magnetic_conversion_time: u64,
    temperature_conversion_time: u64,
}

impl<I: I2c, P: Wait> MLX90393<I, Option<P>>
where
    <I as embedded_hal_async::i2c::ErrorType>::Error: Format,
{
    pub fn new(address: u8, interrupt: Option<P>, i2c: I) -> Self {
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
    ) -> (Option<Status>, [u8; N])
    where
        C: RunCommand<T, M, N>,
    {
        let commands = command.write_command();
        let mut buffer = command.read_buffer();
        let res = self
            .i2c
            .write_read(self.address, &commands, &mut buffer)
            .await;
        match res {
            Ok(_) => {}
            Err(ref err) => {
                debug!("Error: {:#?}", err);
            }
        }
        let status = res.map(|_| Status::from_u8(&buffer[0])).ok();
        //debug!("{:#?}", status);

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
        //debug!("Status: {:#?}", &status);

        (status, buffer)
    }

    pub async fn reset(&mut self) {
        let exit = Command::exit();
        let (_status, _) = self.run_command(exit).await;
        Timer::after_micros(1500).await;
        let reset = Command::reset();
        let (_status, _) = self.run_command(reset).await;
        Timer::after_micros(2000).await;
    }

    pub async fn set_sm<const X: bool, const Y: bool, const Z: bool, const TEMP: bool>(&mut self) {
        //info!("Settings Mode to Single Measurement.");
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
        //debug!("State: {}", self.state);
        self
    }

    pub async fn get_measurement_configuration(&mut self) -> Option<MLXSettings> {
        Timer::after_millis(20).await;
        let data_bits = &self.read_register::<0x00>().await;
        let gain = data_bits.gain();
        let hall_configuration = data_bits.hall_conf()?;
        Timer::after_millis(20).await;

        let data_bits = &self.read_register::<0x02>().await;
        let resolution = data_bits.resolution();
        let magnetic_conversion_time = data_bits.magnetic_axis_conversion_time_micro();
        let temperature_conversion_time = data_bits.temperature_conversion_time_micro();
        Timer::after_millis(20).await;

        let data_bits = &self.read_register::<0x01>().await;
        let temperature_compensation = data_bits.temperature_compensation();
        Timer::after_millis(20).await;

        let data_bits = &self.read_register::<0x24>().await;
        let temp_ref = data_bits.temperature_reference();

        Some(MLXSettings {
            resolution,
            gain,
            hall_configuration,
            temperature_compensation,
            temp_ref,
            magnetic_conversion_time,
            temperature_conversion_time,
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
    ) -> (Option<Status>, MagneticBits) {
        //info!("Waiting for interrupt.");
        if let Some(interrupt) = &mut self.interrupt {
            let _ = interrupt.wait_for_high().await;
            let _ = Timer::after_micros(140).await;
        } else if let Some(state) = self.state {
            let magnetic_axis_count =
                u64::try_from([X, Y, Z].into_iter().filter(|x| *x).count()).unwrap();
            let conversion_time = T_STBY_MICRO
                + T_ACTIVE_MICRO
                + magnetic_axis_count * state.magnetic_conversion_time
                + state.temperature_conversion_time
                + T_CONV_END_MICRO;
            let _ = Timer::after_micros(conversion_time).await;
        }
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
    ) -> (Option<Status>, Option<MagneticField>) {
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
        if let Some(interrupt) = &mut self.interrupt {
            let _ = interrupt.wait_for_high().await;
        }
    }
}
