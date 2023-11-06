use std::{
    io,
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

fn main() {
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
    let rx = read_line_thread(port);
    println!("You have selected {:?}", rx);
    let mut sim_device = Sim868::new(true, GnssConfiguration::default());
    sim_device.start_gnss();
    loop {
        if let Ok(line) = rx.recv() {
            // process receiving data from sim
            println!("{}", line);
            sim_device.process_at(&line);
        }
    }
}

fn read_line_thread(port_name: String) -> std::sync::mpsc::Receiver<String> {
    let (tx, rx) = channel::<String>();

    // perform conncetion to the port

    std::thread::spawn(move || {
        let mut port = serialport::new(&port_name, 115_200)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");
        let mut serial_buf: Vec<u8> = vec![0; 1];
        let mut big_buffer: Vec<u8> = vec![0; 1000];
        loop {
            if port.read_exact(&mut serial_buf).is_ok() {
                big_buffer.push(serial_buf[0]);
                if serial_buf[0] == '\n' as u8 {
                    match std::str::from_utf8(&big_buffer) {
                        Ok(buffer_str) => {
                            if let Some((line, _)) = buffer_str.split_once("\r\n") {
                                tx.send(line.to_string());
                                big_buffer.clear();
                                // return Some(line.into());
                            } else if let Some((line, _)) = buffer_str.split_once('\n') {
                                tx.send(line.to_string());
                                big_buffer.clear();

                                // return Some(line.into());
                            }
                        }
                        Err(e) => {
                            println!(
                                "Error is {} {:?}",
                                e,
                                std::str::from_utf8(
                                    big_buffer
                                        .clone()
                                        .into_iter()
                                        .filter(|c| { c.is_ascii() })
                                        .collect::<Vec<u8>>()
                                        .as_slice()
                                )
                            );
                            big_buffer.clear();
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_micros(500));
        }
    });
    return rx;
}

#[derive(PartialEq)]
enum GnssConfig {
    URC(u8),
    STATUS(bool),
}

#[derive(PartialEq)]
enum GSMConfig {}

struct GnssConfiguration {
    urc: u8,
    urc_enabled: bool,
    power: bool,
    tx: Option<Sender<GnssConfig>>,
}
impl GnssConfiguration {
    fn default() -> GnssConfiguration {
        GnssConfiguration {
            urc: 5,
            urc_enabled: false,
            power: false,
            tx: None,
        }
    }
    fn set_tx(&mut self, tx: Sender<GnssConfig>) {
        self.tx = Some(tx);
    }
}
struct Sim868 {
    power: bool,
    gnss: GnssConfiguration,
    working: bool,
}
impl Sim868 {
    fn new(active: bool, gnss_conf: GnssConfiguration) -> Sim868 {
        Sim868 {
            power: active,
            gnss: GnssConfiguration::default(),
            working: true,
        }
    }
    fn start(&self, rx: Receiver<GSMConfig>) {}

    fn start_gnss(&mut self) {
        let (tx, rx) = channel::<GnssConfig>();
        self.gnss.tx = Some(tx);
        if self.power {
            std::thread::spawn(move || {
                let mut urc = 5;
                loop {
                    match rx.try_recv() {
                        Ok(conf) => match conf {
                            GnssConfig::URC(n) => {
                                // set the sim868 emulator gnss urc
                                urc = n.max(1);
                            }
                            GnssConfig::STATUS(active) => {
                                // stop sending output from this thread
                            }
                        },
                        Err(e) => {}
                    }
                    println!("FROM GNSS THREAD: {}", "AT+UGNSINF=1,2,4,1,2,4,12,4,1");
                    std::thread::sleep(Duration::from_secs(urc as u64));
                }
            });
        } else {
            println!("Failed to run gnss because sim is not activated yet");
        }
    }

    fn process_at(&mut self, at_cmd: &str) {
        if self.power {
            if at_cmd == "AT" {
                println!("AT\\r\\nOK");
            }
        }
    }
}
