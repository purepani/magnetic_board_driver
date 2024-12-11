use std::{
    collections::BTreeMap,
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{self, Stylize},
    terminal,
};
use data_transfer::conversions::{MagneticField, MagneticValue, TempValue};
use postcard::from_bytes;
use ratatui::{
    text::Text,
    widgets::{Paragraph, Row},
    Frame,
};
use serialport::SerialPort;
mod sensor_monitor;
use sensor_monitor::MagneticData;
use tokio::sync::watch::{self, Receiver};

use crate::sensor_monitor::SensorWatcher;

#[derive(Debug, Default, PartialEq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
struct Model {
    pub data: BTreeMap<u8, MagneticData>,
    pub state: RunningState,
}

impl Model {
    pub fn new() -> Self {
        Model::default()
    }

    pub fn modify_from_message(&mut self, message: BTreeMap<u8, MagneticData>) -> &mut Self {
        self.data = message;
        self
    }
}

enum Message {
    RecievedField(BTreeMap<u8, MagneticData>),
    Quit,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    //let mut terminal = ratatui::init();
    //loop {
    //terminal.draw(draw).expect("failed to draw frame");
    //if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
    //break;
    //}
    //}
    //ratatui::restore();
    //let mut stdout = std::io::stdout();
    //let mut buffer = [0; 100000];
    //
    //
    let data = BTreeMap::<u8, MagneticData>::new();
    let (tx, mut rx) = watch::channel(data);
    let watch_sensors = tokio::spawn(async move {
        let mut watcher = SensorWatcher::new();
        loop {
            let vals = watcher.update();
            for (address, val) in vals.await {
                tx.send_if_modified(|all_data| {
                    all_data.insert(address, val);
                    true
                });
            }
        }
    });

    let mut model = Model::new();
    let mut terminal = tui::init_terminal()?;
    //execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    while model.state != RunningState::Done {
        let _ = terminal.draw(|f| view(&mut model, f));
        //sleep(Duration::new(0, 100000000));
        //let val = data_transfer::messaging::Message::read(&mut port);
        //let mut current_msg = val.ok().map(|msg| Message::RecievedField(msg));
        let mut current_msg = handle_event(&model, &mut rx)?;
        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }
    watch_sensors.abort();
    tui::restore_terminal()?;
    Ok(())
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    let block = ratatui::widgets::List::new(["test", "test2"]);
    let rows = [Row::new(vec!["X", "Y", "Z", "TEMP"])];
    let table = ratatui::widgets::Table::new(rows, [15, 15, 15, 15]);
    frame.render_widget(table, frame.area());
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::RecievedField(field) => {
            model.modify_from_message(field);
            None
        }
        Message::Quit => {
            model.state = RunningState::Done;
            None
        }
    }
}

fn handle_event(
    model: &Model,
    rx: &mut Receiver<BTreeMap<u8, MagneticData>>,
) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    if rx.has_changed()? {
        let val = rx.borrow_and_update();
        return Ok(Some(Message::RecievedField(val.clone())));
    }

    Ok(None)
}
fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn view(model: &mut Model, frame: &mut Frame) {
    let fields = model.data.iter();
    //Row::new(vec!["X", "Y", "Z", "TEMP"]),
    let rows = fields
        .map(|(address, data)| {
            let [x, y, z] = [data.field.x, data.field.y, data.field.z].map(|field| {
                field.map_or("0".to_string(), |val| match val {
                    data_transfer::conversions::MagneticValue::uT(x) => format!("{:.3}", x),
                })
            });
            Row::new(vec![address.to_string(), x, y, z, data.time.to_string()])
        })
        .collect::<Vec<_>>();
    let table = ratatui::widgets::Table::new(rows, [15, 15, 15, 15, 15]);
    //frame.render_widget(Paragraph::new(format!("Magnetic fields")), frame.area());
    //frame.render_widget(
    //Paragraph::new(format!("Magnetic fields empty: {}", model.data.is_empty())),
    //frame.area(),
    //);
    frame.render_widget(table, frame.area());
}

mod tui {
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        Terminal,
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
