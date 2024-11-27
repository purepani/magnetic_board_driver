use std::{
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue,
    style::{self, Stylize},
    terminal,
};
use data_transfer::conversions::{MagneticField, MagneticValue, TempValue};
use postcard::from_bytes;
use ratatui::{text::Text, widgets::Row, Frame};
use serialport::SerialPort;

struct Model {
    pub fields: MagneticField,
}

enum Message {
    RecievedField(MagneticField),
}

fn main() -> io::Result<()> {
    //let mut terminal = ratatui::init();
    //loop {
    //terminal.draw(draw).expect("failed to draw frame");
    //if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
    //break;
    //}
    //}
    //ratatui::restore();
    let mut stdout = std::io::stdout();
    let p = serialport::available_ports().expect("Serial Port not found");
    let baud_rate = 115200;
    let port_builder = serialport::new(p.first().unwrap().port_name.clone(), baud_rate);
    let mut port = port_builder.open().unwrap();
    let _ = port.write(&[1; 8]);
    //let mut buffer = [0; 100000];
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    loop {
        let val = data_transfer::messaging::Message::read(&mut port).map(|msg| msg);
        match val {
            Ok(msg) => {
                let (bx, by, bz, temp) = (
                    msg.field.x.unwrap_or(MagneticValue::uT(0.0)),
                    msg.field.y.unwrap_or(MagneticValue::uT(0.0)),
                    msg.field.z.unwrap_or(MagneticValue::uT(0.0)),
                    msg.field.t.unwrap_or(TempValue::Celsius(0.0)),
                );
                let (x, y, z) = msg.position;
                let val = format!(
                    "x: {:.2}\ty: {:.2}\tz: {:.2}\nBx: {:.3}\tBy: {:.3}\tBz: {:.3}\tTemp: {:.3}\n",
                    x,
                    y,
                    z,
                    bx.value(),
                    by.value(),
                    bz.value(),
                    temp.value()
                );
                queue!(
                    stdout,
                    cursor::MoveToPreviousLine(2),
                    style::PrintStyledContent(val.magenta())
                )?;
            }
            Err(val) => {
                println!("{:#?}", val);
            }
        };
        port.clear(serialport::ClearBuffer::Input);
        sleep(Duration::new(0, 100000000));
    }
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    let block = ratatui::widgets::List::new(["test", "test2"]);
    let rows = [Row::new(vec!["X", "Y", "Z", "TEMP"])];
    let table = ratatui::widgets::Table::new(rows, [15, 15, 15, 15]);
    frame.render_widget(table, frame.area());
}

fn test() {
    let p = serialport::available_ports().expect("Serial Port not found");
    let baud_rate = 115200;
    let port_builder = serialport::new(p.first().unwrap().port_name.clone(), baud_rate);
    let mut port = port_builder.open().unwrap();
    let _ = port.write(&[1; 8]);
    let mut buffer = [0; 100000];
    let val = postcard::from_io::<MagneticField, _>((port, &mut buffer));
    match val {
        Ok((field, _)) => print!("{:#?}", field),
        Err(val) => print!("{:#?}", val),
    };
}

fn update(model: &Model, msg: Message) -> Model {
    match msg {
        Message::RecievedField(field) => Model { fields: field },
    }
}

fn view(model: &Model, frame: &mut Frame) {
    let field = model.fields;
    let [x, y, z] = [field.x, field.y, field.z].map(|field| {
        field.map_or("0".to_string(), |val| match val {
            data_transfer::conversions::MagneticValue::uT(x) => x.to_string(),
        })
    });
    let rows = [
        Row::new(vec!["X", "Y", "Z", "TEMP"]),
        Row::new(vec![x, y, z]),
    ];
    let table = ratatui::widgets::Table::new(rows, [15, 15, 15, 15]);
    frame.render_widget(table, frame.area())
}
