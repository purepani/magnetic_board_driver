use crate::app::{Context, SpawnCtx, AppTx};
use defmt::info;
use postcard_rpc::header::VarHeader;
use postcard_rpc::server::Sender;
use data_transfer::rpc::SensorField;
use embassy_executor;
use portable_atomic::{AtomicBool, Ordering};

pub fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    info!("ping");
    rqst
}

pub async fn single_request_handler(context: &mut Context, _header: VarHeader, rqst: (u32, u32)) -> SensorField {
    
    let board_index = usize::try_from(rqst.0).or(Err(())).unwrap();
    let sensor_index = usize::try_from(rqst.1).or(Err(())).unwrap();
    let sensor = context.sensor_groups.get_mut(board_index).ok_or(()).unwrap().get_mut();
    let message = sensor.get_message(sensor_index).await;
    info!("{}", message);
    message.unwrap()
}

static STOP: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
async fn stream_field(
    context: SpawnCtx<1>,
    header: VarHeader,
    rqst: (),
    sender: Sender<AppTx>,
) {
    
}


 
