use std::{
    error::Error,
    io,
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

mod sim868;
mod ui;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    let ports = serialport::available_ports().expect("No ports found!");
    println!("Please select your device connected port");
    for (index, p) in ports.iter().enumerate() {
        println!("{}: {}", index, p.port_name);
    }
    let mut chosen_port = String::new();
    io::stdin()
        .read_line(&mut chosen_port)
        .expect("Expected your input");

    let port = ports[chosen_port[0..chosen_port.len() - 1]
        .parse::<usize>()
        .expect("Not a valid number")]
    .port_name
    .clone();
    let (port_tx, port_rx) = channel::<String>();
    let rx = utils::serial::read_line_thread(port, port_rx);
    // println!("You have selected {:?}", rx);
    // let mut sim_device = Sim868::new(true, GnssConfiguration::default());
    // sim_device.start_gnss();
    // loop {
    //     if let Ok(line) = rx.recv() {
    //         // process receiving data from sim
    //         println!("{}", line);
    //         sim_device.process_at(&line);
    //     }
    // }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = ui::run_ui(&mut terminal, rx, port_tx);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
