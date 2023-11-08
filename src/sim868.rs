use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(PartialEq)]
pub enum GnssConfig {
    URC(u8),
    STATUS(bool),
}

const AT_CFUN: &str = "AT+CFUN";
const AT_IPR: &str = "AT+IPR";
const AT_AND_W: &str = "AT&W";
const CME_ERROR: &str = "+CME ERROR:0";
const OK: &str = "\r\nOK\r\n";

#[derive(PartialEq)]
pub enum GSMConfig {}

pub struct GnssConfiguration {
    pub urc: u8,
    pub urc_enabled: bool,
    pub power: bool,
    // pub tx: Option<Sender<GnssConfig>>,
}
impl GnssConfiguration {
    pub fn default() -> GnssConfiguration {
        GnssConfiguration {
            urc: 5,
            urc_enabled: false,
            power: false,
            // tx: None,
        }
    }
    pub fn set_tx(&mut self, tx: Sender<GnssConfig>) {
        // self.tx = Some(tx);
    }
    pub fn power(&self) -> bool {
        self.power
    }
}
pub struct Sim868 {
    pub power: bool,
    pub gnss: Arc<Mutex<GnssConfiguration>>,
    pub working: bool,
    // pub port_tx: Option<Sender<String>>,
}
impl Sim868 {
    pub fn new(active: bool, gnss_conf: GnssConfiguration) -> Sim868 {
        Sim868 {
            power: active,
            gnss: Arc::new(Mutex::new(GnssConfiguration::default())),
            working: true,
            // port_tx: (Some(port_tx)),
        }
    }
    // pub fn write_to_port(&self, data: &str) {
    //     if let Some(locked_tx) = &self.port_tx {
    //         locked_tx.send(data.to_owned());
    //     }
    // }
    pub fn start(&self, rx: Receiver<GSMConfig>) {}

    pub fn start_gnss(&mut self, port_tx: Sender<String>) -> Sender<GnssConfig> {
        let (tx, rx) = channel::<GnssConfig>();
        // self.gnss.tx = Some(tx.clone());
        let shared_self = self.gnss.clone(); //Arc::new(Mutex::new(self.gnss));

        if self.power {
            std::thread::spawn(move || {
                let mut urc = 5;
                loop {
                    {
                        if shared_self.lock().unwrap().power {
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

                            port_tx
                                .send("AT+UGNSINF=1,2,4,1,2,4,12,4,1".to_owned())
                                .unwrap();
                            std::thread::sleep(Duration::from_secs(urc as u64));
                        }
                    }
                }
            });
        } else {
            println!("Failed to run gnss because sim is not activated yet");
        }
        tx
    }

    pub fn process_at(&mut self, at_cmd: &str) -> Option<Vec<String>> {
        // if self.power {
        // if at_cmd.trim() == "AT" {
        if at_cmd.len() <= 2 {
            return Some(vec!["AT\r\nOK".to_owned()]);
        } else if at_cmd.starts_with(AT_CFUN) {
            let t = at_cmd.as_bytes()[AT_CFUN.len()] as char;
            if t == '=' {
                let func = at_cmd.as_bytes()[AT_CFUN.len() + 1] as char;
                let rst = at_cmd.as_bytes()[AT_CFUN.len() + 3] as char;
                return Some(
                    vec![
                        "AT+CFUN=1,1\r\r\nOK\r\n".to_owned(),
                        "\r\nRDY\r\n\r\n+CFUN: 1\r\n\r\n+CREG: 0\r\n\r\n+CPIN: READY\r\n\r\n+CGREG:2\r\n\r\nCall Ready\r\n".to_owned() 
                ]);
            } else if t == '?' {
                return Some(vec![t.to_string()]);
            } else {
                return Some(vec![CME_ERROR.to_owned()]);
            }
        } else if at_cmd.starts_with(AT_IPR) {
            return Some(vec![at_cmd.to_owned() + "\r", OK.to_owned()]);
        } else if at_cmd.starts_with(AT_AND_W) {
            return Some(vec![at_cmd.to_owned() + "\r", OK.to_owned()]);
        } else {
            return Some(vec!["Not Implemendted Command".to_owned() + at_cmd]);
        }
    }
}
