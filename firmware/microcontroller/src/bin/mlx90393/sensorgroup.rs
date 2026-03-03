use data_transfer::{
    conversions::MagneticField,
    messaging::{self, Writable},
};
use defmt::Format;
use defmt::{debug, info};
use embassy_stm32::exti::ExtiInput;
use embassy_time::{Instant, Timer};
use embedded_hal_async::{digital::Wait, i2c::I2c};
use embedded_io::Write;

use super::sensor::{Status, MLX90393};

pub struct Sensor<I, P> {
    pub position: (f32, f32, f32),
    mlx: MLX90393<I, P>,
}

#[derive(Debug, Format)]
pub enum Error {
    DataError(messaging::Error),
    StatusError(Status),
}

impl From<messaging::Error> for Error {
    fn from(value: messaging::Error) -> Self {
        Self::DataError(value)
    }
}

impl<I: I2c, P: Wait> Sensor<I, Option<P>> {
    pub async fn new(address: u8, i2c: I, position: (f32, f32, f32)) -> Self
where {
        let mlx = MLX90393::new(address, None, i2c);
        Self::from_mlx(mlx, position).await
    }

    pub async fn from_mlx(mlx: MLX90393<I, Option<P>>, position: (f32, f32, f32)) -> Self {
        let mut sensor = Self { mlx, position };
        sensor.mlx.reset().await;
        Timer::after_micros(500).await;
        sensor.mlx.set_measurement_configuration().await;
        //sensor.mlx.set_burst::<true, true, true, true>().await;
        sensor
    }

    pub async fn send_message<W: embedded_io_async::Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<messaging::Message, Error> {
        self.mlx
            .set_single_measurmenet::<true, true, true, true>()
            .await;
        //Timer::after_millis(50).await;
        let (status, field) = self.mlx.get_field::<true, true, true, true>().await;
        //if status.is_some_and(|val| !val.burst_mode) {
        //self.mlx.set_burst::<true, true, true, true>().await;
        //}
        if status.error {
            return Err(Error::StatusError(status));
        }
        let time = Instant::now().as_micros();
        let message =
            field.map(|f| messaging::Message::new(f, self.position, self.mlx.address, time));
        let message = message.ok_or(data_transfer::messaging::Error::FailedRead)?;
        message.write_to(writer).await?;
        Ok(message)
    }

    pub async fn get_message(&mut self) -> Result<messaging::Message, ()> {
        self.mlx
            .set_single_measurmenet::<true, true, true, true>()
            .await;
        Timer::after_millis(50).await;
        let (status, field) = self.mlx.get_field::<true, true, true, true>().await;
        //debug!("{:#?}", status);
        //if status.is_some_and(|val| !val.burst_mode) {
        //self.mlx.set_burst::<true, true, true, true>().await;
        //}
        let time = Instant::now().as_micros();
        let message =
            field.map(|f| messaging::Message::new(f, self.position, self.mlx.address, time));
        message.ok_or(())
    }

    pub async fn set_burst_mode(&mut self) {
        self.mlx.set_burst::<true, true, true, true>().await
    }
}

impl<'a, I: I2c> Sensor<I, Option<ExtiInput<'a>>>
where
    <I as embedded_hal_async::i2c::ErrorType>::Error: Format,
{
    pub async fn new_stm(address: u8, position: (f32, f32, f32), i2c: I) -> Self {
        Self::new(address, i2c, position).await
    }
}

trait SendValues {
    async fn send_message<W: Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<MagneticField, data_transfer::messaging::Error>;
}

pub struct SensorBuilder {
    pub address: u8,
    pub position: (f32, f32, f32),
}

impl<'a> SensorBuilder {
    pub fn new_stm(address: u8, position: (f32, f32, f32)) -> Self {
        Self { address, position }
    }

    pub async fn with_i2c<I: I2c>(self, i2c: I) -> Sensor<I, Option<ExtiInput<'a>>>
    where
        <I as embedded_hal_async::i2c::ErrorType>::Error: Format,
    {
        Sensor::new_stm(self.address, self.position, i2c).await
    }
}
