use crate::app::Context;
use defmt::info;
use postcard_rpc::header::VarHeader;
use data_transfer::rpc::SensorField;

pub fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    info!("ping");
    rqst
}

pub async fn single_request_handler(context: &mut Context, _header: VarHeader, rqst: (u32, u32)) -> SensorField {
    let board_index = usize::try_from(rqst.0).or(Err(())).unwrap();
    let sensor_index = usize::try_from(rqst.1).or(Err(())).unwrap();
    let sensor = context.sensor_groups.get_mut(board_index).ok_or(()).unwrap();
    let message = sensor.get_message(sensor_index).await;
    info!("{}", message);
    message.unwrap()
}

