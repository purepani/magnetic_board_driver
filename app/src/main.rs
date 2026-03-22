use std::{collections::BTreeMap, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use postcard_rpc::host_client::HostClient;
use postcard_rpc::standard_icd::WireError;
use ratatui::{text::Text, widgets::Row, Frame};
mod sensor_monitor;
use sensor_monitor::MagneticData;
use tokio::sync::watch::{self, Receiver};

use crate::sensor_monitor::SensorWatcher;
use iced::widget::{button, column, combo_box, container, row, text, text_input};
use iced::{Element, Task};
use std::fmt::{Display, format};

use data_transfer::{
    self,
    messaging::MessageReader,
    rpc::{SensorField, SingleFieldValue},
};

#[derive(Debug, Clone)]
struct SerialPortInfo(serialport::SerialPortInfo);

impl Display for SerialPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.port_name)
    }
}

#[derive(Debug, Clone)]
enum Message {
    PortSelected(SerialPortInfo),
    UpdatePorts,
    UpdatePingBoard(String),
    UpdatePingSensor(String),
    GetField,
    RecievedField(data_transfer::rpc::SensorField),
}

#[derive(Debug, Clone, Default)]
struct PingArgs {
    board: String,
    sensor: String,
}

#[derive(Default)]
struct Context {
    sensor_watcher: Option<SensorWatcher>,
    sensors: combo_box::State<SerialPortInfo>,
    ping_args: PingArgs,
    ping_field: Option<SensorField>,
}

async fn get_single_value(board: u32, sensor: u32, client: HostClient<WireError>) -> SensorField {
    let args = (board, sensor);
    client.send_resp::<SingleFieldValue>(&args).await.unwrap()
}

fn update(context: &mut Context, message: Message) -> Task<Message> {
    match message {
        Message::PortSelected(serial_port_info) => {
            context.sensor_watcher = Some(SensorWatcher::new(&serial_port_info));
            Task::none()
        }
        Message::UpdatePorts => {
            let ports = serialport::available_ports()
                .unwrap_or(Vec::new())
                .into_iter()
                .map(SerialPortInfo)
                .collect();
            let port_info = context.sensor_watcher.as_ref().map(|s| s.port_info());
            context.sensors = combo_box::State::<_>::with_selection(ports, port_info);
            Task::none()
        }
        Message::UpdatePingBoard(s) => {
            context.ping_args.board = s;
            Task::none()
        }
        Message::UpdatePingSensor(s) => {
            context.ping_args.sensor = s;
            Task::none()
        }
        Message::GetField => {
            if let Some(sw) = &context.sensor_watcher {
                let board = &context.ping_args.board;
                let sensor = &context.ping_args.sensor;
                let board = board.parse().unwrap_or(0);
                let sensor = sensor.parse().unwrap_or(0);

                let client = sw.get_client();
                Task::perform(
                    get_single_value(board, sensor, client), 
                    Message::RecievedField,
                )
            } else {
                Task::none()
            }
        }
        Message::RecievedField(sensor_field) => {
            context.ping_field = Some(sensor_field);
            Task::none()
        },
    }
}

fn view(context: &Context) -> Element<'_, Message> {
    let serial_selector = combo_box(
        &context.sensors,
        "Select Sensor",
        context.sensor_watcher.as_ref().map(|s| s.port_info()),
        Message::PortSelected,
    )
    .on_open(Message::UpdatePorts);

    let ping_widget = container(row![
        text("Board Number: "),
        text_input("Board Number", &context.ping_args.board).on_input(Message::UpdatePingBoard),
        text("Sensor Number: "),
        text_input("Sensor Number", &context.ping_args.sensor).on_input(Message::UpdatePingSensor),
        button("Get Field Value").on_press(Message::GetField),
        text(format!("x: {:.2}", context.ping_field.as_ref().map(|f| f.field.x.map(|x| x.value()).unwrap_or(0.0)).unwrap_or(0.0))),
        text(format!("y: {:.2}", context.ping_field.as_ref().map(|f| f.field.y.map(|y| y.value()).unwrap_or(0.0)).unwrap_or(0.0))),
        text(format!("z: {:.2}", context.ping_field.as_ref().map(|f| f.field.z.map(|z| z.value()).unwrap_or(0.0)).unwrap_or(0.0))),
    ]);

    column![serial_selector, ping_widget].padding(10).into()
}

#[tokio::main]
pub async fn main() -> iced::Result {
    iced::run(update, view)
}
