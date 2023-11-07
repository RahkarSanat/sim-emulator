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

    pub fn process_at(&mut self, at_cmd: &str) -> Option<String> {
        if self.power {
            if at_cmd == "AT" {
                return Some("AT\r\nOK".to_owned());
            }
        }
        None
    }
}
