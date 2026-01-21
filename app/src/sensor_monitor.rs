use std::time::Duration;

use data_transfer::{
    self,
    messaging::MessageReader,
};

pub struct SensorWatcher<P = ()> {
    port: P,
    message_reader: MessageReader,
}

#[derive(Debug, Clone)]
pub struct MagneticData {
    pub field: data_transfer::conversions::MagneticField,
    pub position: (f32, f32, f32),
    pub time: u64,
}

impl MagneticData {
    pub fn from_message(message: &data_transfer::messaging::Message) -> Self {
        Self {
            field: message.field,
            position: message.position,
            time: message.time,
        }
    }
}

impl SensorWatcher {
    pub fn new() -> SensorWatcher<Box<dyn serialport::SerialPort + 'static>> {
        let p = serialport::available_ports().expect("Serial Port not found");
        let baud_rate = 115200;
        let port_builder = serialport::new(p.first().unwrap().port_name.clone(), baud_rate);
        let mut port = port_builder.open().unwrap();
        let message_reader = MessageReader::new();
        let _ = port.write(&[1; 8]);
        SensorWatcher {
            port,
            message_reader,
        }
    }
}

impl<P: std::io::Read> SensorWatcher<P> {
    pub async fn update(&mut self) -> Vec<(u8, MagneticData)> {
        tokio::time::sleep(Duration::from_micros(100)).await;
        //let parsed_values = data_transfer::messaging::Message::read_all(&mut self.port);
        let parsed_values = self.message_reader.read_all(&mut self.port);
        
        parsed_values
            .into_iter()
            .filter_map(|msg| {
                let msg = msg.ok()?;
                Some((
                    msg.address,
                    MagneticData {
                        field: msg.field,
                        position: msg.position,
                        time: msg.time,
                    },
                ))
            })
            .collect()
    }
    pub fn watcher(&mut self) {}
}
