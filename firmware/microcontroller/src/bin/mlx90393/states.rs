use embedded_hal_async::i2c::I2c;

use crate::mlx90393::commands::CommandData;

//enum Event {
//Command(CommandData),
//MeasurementReady,
//}

trait IsSensorMode {
    fn new() -> Self;
}

pub struct NoMode;
pub struct Burst;
pub struct WakeOnChange;
pub struct SingleMeasurement;

trait IsMode {
    fn new() -> Self;
}
impl IsMode for NoMode {
    fn new() -> Self {
        Self
    }
}
impl IsMode for Burst {
    fn new() -> Self {
        Self
    }
}
impl IsMode for WakeOnChange {
    fn new() -> Self {
        Self
    }
}
impl IsMode for SingleMeasurement {
    fn new() -> Self {
        Self
    }
}

pub struct SensorState<S, T> {
    pub(crate) state: S,
    pub(crate) mode: T,
}

pub struct Idle;
pub struct Measuring;
pub struct Measured;

impl<T: IsMode> From<SensorState<Idle, NoMode>> for SensorState<Measuring, T> {
    fn from(value: SensorState<Idle, NoMode>) -> Self {
        Self {
            state: Measuring,
            mode: T::new(),
        }
    }
}

impl<T> From<SensorState<Measuring, T>> for SensorState<Measured, T> {
    fn from(value: SensorState<Measuring, T>) -> Self {
        Self {
            state: Measured,
            mode: value.mode,
        }
    }
}

impl<T> From<SensorState<Measured, T>> for SensorState<Idle, NoMode> {
    fn from(value: SensorState<Measured, T>) -> Self {
        Self {
            state: Idle,
            mode: NoMode,
        }
    }
}

impl From<SensorState<Measured, Burst>> for SensorState<Measuring, Burst> {
    fn from(value: SensorState<Measured, Burst>) -> Self {
        Self {
            state: Measuring,
            mode: Burst,
        }
    }
}

impl From<SensorState<Measured, WakeOnChange>> for SensorState<Measuring, WakeOnChange> {
    fn from(value: SensorState<Measured, WakeOnChange>) -> Self {
        Self {
            state: Measuring,
            mode: WakeOnChange,
        }
    }
}

impl<T> From<SensorState<Measured, T>> for SensorState<Measuring, SingleMeasurement> {
    fn from(value: SensorState<Measured, T>) -> Self {
        Self {
            state: Measuring,
            mode: SingleMeasurement,
        }
    }
}
