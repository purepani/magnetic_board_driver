use data_transfer::{conversions::MagneticField, messaging::Message};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Pin, Pull},
    Peripheral,
};
use embassy_time::Timer;
use embedded_hal_async::{digital::Wait, i2c::I2c};
use embedded_io::Write;

use super::sensor::MLX90393;

pub struct Sensor<I, P> {
    pub position: (f32, f32, f32),
    mlx: MLX90393<I, P>,
}

impl<I: I2c, P: Wait> Sensor<I, P> {
    pub async fn new(address: u8, interrupt: P, i2c: I, position: (f32, f32, f32)) -> Self {
        let mlx = MLX90393::new(address, interrupt, i2c);

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

    pub async fn send_message<W: Write>(
        &mut self,
        writer: &mut W,
    ) -> Result<MagneticField, data_transfer::messaging::Error> {
        self.mlx
            .set_single_measurmenet::<true, true, true, true>()
            .await;
        let field = self
            .mlx
            .get_field::<true, true, true, true>()
            .await
            .1
            .unwrap_or_default();
        let message = Message::new(field, self.position);
        message.write_to(writer)?;
        Ok(field)
    }
}

impl<'a, I: I2c, T: Pin> Sensor<I, ExtiInput<'a, T>> {
    pub async fn new_stm(
        address: u8,
        position: (f32, f32, f32),
        pin: impl Peripheral<P = T> + 'a,
        ch: impl Peripheral<P = T::ExtiChannel> + 'a,
        i2c: I,
    ) -> Self
where {
        let pin = Input::new(pin, Pull::Down);
        let interr = ExtiInput::new(pin, ch);
        Self::new(address, interr, i2c, position).await
    }
}
