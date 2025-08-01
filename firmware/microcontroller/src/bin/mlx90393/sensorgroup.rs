use data_transfer::{
    conversions::MagneticField,
    messaging::{self, Writable},
};
use embassy_stm32::{
    exti::{AnyChannel, Channel, ExtiInput},
    gpio::{AnyPin, Input, Pin, Pull},
    Peripheral,
};
use embassy_time::{Instant, Timer};
use embedded_hal_async::{digital::Wait, i2c::I2c};
use embedded_io::Write;

use super::sensor::MLX90393;

pub struct Sensor<I, P> {
    pub position: (f32, f32, f32),
    mlx: MLX90393<I, P>,
}

impl<I: I2c, P: Wait> Sensor<I, Option<P>> {
    pub async fn new(address: u8, interrupt: Option<P>, i2c: I, position: (f32, f32, f32)) -> Self {
        let mlx = MLX90393::new(address, interrupt, i2c);
        Self::from_mlx(mlx, position).await
    }

    pub async fn from_mlx(mlx: MLX90393<I, Option<P>>, position: (f32, f32, f32)) -> Self {
        let mut sensor = Self { mlx, position };
        Timer::after_millis(100).await;
        sensor.mlx.reset().await;
        Timer::after_millis(100).await;
        sensor.mlx.set_measurement_configuration().await;
        Timer::after_millis(100).await;
        //sensor.mlx.set_burst::<true, true, true, true>().await;
        Timer::after_millis(100).await;
        sensor
    }

    pub async fn send_message<W: embedded_io_async::Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<messaging::Message, data_transfer::messaging::Error> {
        self.mlx
            .set_single_measurmenet::<true, true, true, true>()
            .await;
        let field = self
            .mlx
            .get_field::<true, true, true, true>()
            .await
            .1
            .unwrap_or_default();
        let time = Instant::now().as_micros();
        let message = messaging::Message::new(field, self.position, self.mlx.address, time);
        message.write_to(writer).await?;
        Ok(message)
    }

    pub async fn get_message(
        &mut self,
    ) -> Result<messaging::Message, data_transfer::messaging::Error> {
        self.mlx
            .set_single_measurmenet::<true, true, true, true>()
            .await;
        let field = self
            .mlx
            .get_field::<true, true, true, true>()
            .await
            .1
            .unwrap_or_default();
        let time = Instant::now().as_ticks();
        let message = messaging::Message::new(field, self.position, self.mlx.address, time);
        Ok(message)
    }
}

impl<'a, I: I2c> Sensor<I, Option<ExtiInput<'a>>> {
    pub async fn new_stm<T: Pin>(
        address: u8,
        position: (f32, f32, f32),
        pin_ch: Option<(
            impl Peripheral<P = T> + 'a,
            impl Peripheral<P = T::ExtiChannel> + 'a,
        )>,
        i2c: I,
    ) -> Self {
        let interr = if let Some((pin, ch)) = pin_ch {
            Some(ExtiInput::new(pin, ch, Pull::Down))
        } else {
            None
        };
        Self::new(address, interr, i2c, position).await
    }
}

trait SendValues {
    async fn send_message<W: Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<MagneticField, data_transfer::messaging::Error>;
}

pub struct SensorBuilder<P, Ch> {
    pub address: u8,
    pub position: (f32, f32, f32),
    pub pin_ch: Option<(P, Ch)>,
}

impl<'a, P: Pin, Ch: Channel> SensorBuilder<P, Ch>
where
    P: Peripheral<P = P> + 'a,
    Ch: Peripheral<P = P::ExtiChannel> + 'a,
{
    pub fn new_stm_with_interrupt(address: u8, position: (f32, f32, f32), pin: P, ch: Ch) -> Self {
        Self {
            address,
            position,
            pin_ch: Some((pin, ch)),
        }
    }

    pub fn new_stm(address: u8, position: (f32, f32, f32)) -> Self {
        Self {
            address,
            position,
            pin_ch: None,
        }
    }

    pub fn new_stm_degraded(
        address: u8,
        position: (f32, f32, f32),
        pin: P,
        ch: Ch,
    ) -> SensorBuilder<AnyPin, AnyChannel> {
        let pin = pin.degrade();
        let ch = ch.degrade();
        SensorBuilder {
            address,
            position,
            pin_ch: Some((pin, ch)),
        }
    }
    pub async fn with_i2c<I: I2c>(self, i2c: I) -> Sensor<I, Option<ExtiInput<'a>>> {
        Sensor::new_stm(self.address, self.position, self.pin_ch, i2c).await
    }
}
