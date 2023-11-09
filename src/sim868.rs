use std::{
    str,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

#[macro_export]
macro_rules! at {
    ($cmd: expr) => {
        format!("\r\n{}\r\n", $cmd).to_owned()
    };
}

#[derive(PartialEq)]
pub enum GnssConfig {
    URC(u8),
    STATUS(bool),
}

const AT_CFUN: &str = "AT+CFUN";
const AT_IPR: &str = "AT+IPR";
const AT_AND_W: &str = "AT&W";
const CME_ERROR: &str = "+CME ERROR:0";
const OK: &str = "OK";
const AT_ECHO: &str = "ATE";
const AT_CMEE: &str = "AT+CMEE";
const AT_CGMI: &str = "AT+CGMI";
const AT_CGMM: &str = "AT+CGMM";
const AT_CGSN: &str = "AT+CGSN";
const AT_CGMR: &str = "AT+CGMR";
const AT_CREG: &str = "AT+CREG";

#[derive(PartialEq)]
pub struct GSMConfig {
    baudrate: usize,
    echo: bool,
    cmee: u8,
    fun_mode: Option<u8>,
    rst_mod: Option<u8>,
}

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
    pub configs: GSMConfig,
    pub reg_status: Arc<Mutex<u8>>,
    // pub baudrate: usize, // pub port_tx: Option<Sender<String>>,
}
impl Sim868 {
    pub fn new(active: bool, gnss_conf: GnssConfiguration) -> Sim868 {
        Sim868 {
            power: active,
            gnss: Arc::new(Mutex::new(GnssConfiguration::default())),
            reg_status: Arc::new(Mutex::new(0)),
            working: true,
            configs: GSMConfig {
                baudrate: 115200,
                echo: false,
                fun_mode: None,
                rst_mod: None,
                cmee: 0,
            },
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

    pub fn process_at(&mut self, at_cmd: &str, tx: Sender<String>) -> Option<Vec<String>> {
        // if self.power {
        // if at_cmd.trim() == "AT" {
        let mut res = vec![];
        if self.configs.echo {
            res.push(at_cmd.to_owned() + "\r");
        }

        if at_cmd.len() <= 2 {
            return Some(vec!["AT\r\nOK".to_owned()]);
        } else if at_cmd.starts_with(AT_CFUN) {
            let t = at_cmd.as_bytes()[AT_CFUN.len()] as char;
            if t == '=' {
                let func = at_cmd.as_bytes()[AT_CFUN.len() + 1] as u8 - '0' as u8;
                if func == 0 || func == 1 || func == 4 {
                    self.configs.fun_mode = Some(func);
                    if func == 1 {
                        res.push(at!("+CREG: 0"));
                        res.push(at!("+CPIN: READY"));
                        res.push(at!("+CGREG:2"));
                        res.push(at!("Call Ready"));
                        res.push(at!(OK));
                    }
                }
                if at_cmd.len() > AT_CFUN.len() + 2 {
                    let rst = at_cmd.as_bytes()[AT_CFUN.len() + 3] as u8 - '0' as u8;
                    self.configs.fun_mode = Some(rst);
                    if rst == 1 {
                        res.push(at!("+CFUN=1"));
                    }
                }
                return Some(res);
            } else if t == '?' {
                return Some(vec![t.to_string()]);
            } else {
                return Some(vec![CME_ERROR.to_owned()]);
            }
        } else if at_cmd.starts_with(AT_IPR) {
            res.push(at!(OK));
            // vec![self.ipr(at_cmd).to_owned()
            return Some(res);
        } else if at_cmd.starts_with(AT_AND_W) {
            res.push(at!(OK));
            return Some(res);
        } else if at_cmd.starts_with(AT_ECHO) {
            res.push(self.echo(at_cmd));
            return Some(res);
        } else if at_cmd.starts_with(AT_CMEE) {
            res.push(self.cmee(at_cmd));
            return Some(res);
        } else if at_cmd.starts_with(AT_CGMI)
            || at_cmd.starts_with(AT_CGMM)
            || at_cmd.starts_with(AT_CGSN)
            || at_cmd.starts_with(AT_CGMR)
        {
            res.push(self.manu_info(at_cmd));
            res.push(at!(OK).to_owned());
            return Some(res);
        } else if at_cmd.starts_with(AT_CREG) {
            self.creg_thread(tx.clone());
            res.push(self.cmee(at_cmd));
            res.push(at!(OK));
            return Some(res);
        } else {
            return Some(vec!["Not Implemendted Command".to_owned() + at_cmd]);
        }
    }
}

pub mod sim {

    pub mod parse {
        use crate::sim868::{
            Sim868, AT_CGMI, AT_CGMM, AT_CGMR, AT_CGSN, AT_CMEE, AT_ECHO, AT_IPR, CME_ERROR, OK,
        };
        impl Sim868 {
            pub fn ipr<'a>(&mut self, line: &'a str) -> &'a str {
                let op = line.as_bytes()[AT_IPR.len()] as char;
                if op == '=' {
                    //set device baudrate
                    let requested_baudrate = line[AT_IPR.len() + 1..].parse::<usize>().unwrap();
                    if requested_baudrate == self.configs.baudrate {
                        return line;
                    } else {
                        return CME_ERROR;
                    }
                    // if possible, then
                } else if op == '?' {
                    // return the baudrate
                    return "";
                } else {
                    return CME_ERROR;
                }
            }

            pub fn echo(&mut self, line: &str) -> String {
                let mode = line.as_bytes()[AT_ECHO.len()] as u8 - '0' as u8;
                if mode == 1 {
                    self.configs.echo = true;
                    return at!(OK).to_owned();
                }
                return line.to_string();
            }

            pub fn cmee(&mut self, line: &str) -> String {
                let mode = line.as_bytes()[AT_CMEE.len()] as char;
                if mode == '=' {
                    let value = line.as_bytes()[AT_CMEE.len() + 1] as u8 - '0' as u8;
                    self.configs.cmee = value;
                    return at!(OK).to_owned();
                }
                return mode.to_string();
            }

            pub fn manu_info(&mut self, line: &str) -> String {
                let result = if line == AT_CGMI {
                    at!("SIMCOM_Ltd")
                } else if line == AT_CGMM {
                    at!("SIMCOM_SIM868")
                } else if line == AT_CGSN {
                    at!("86737803397915")
                } else if line == AT_CGMR {
                    at!("1418B05Scustome")
                } else {
                    at!(CME_ERROR)
                };
                result
            }
            pub fn creg(&mut self, line: &str) -> String {
                let mode = line.as_bytes()[AT_CMEE.len()] as char;
                if mode == '=' {
                    let value = line.as_bytes()[AT_CMEE.len() + 1] as u8 - '0' as u8;
                    if value == 1 {
                        // spawn thread to network registration data
                    }
                    self.configs.cmee = value;
                    return at!(OK).to_owned();
                }
                return mode.to_string();
            }
        }
    }

    pub mod urc {
        use std::{sync::mpsc::Sender, time::Duration};

        use crate::sim868::Sim868;

        impl Sim868 {
            pub fn creg_thread(&self, tx: Sender<String>) {
                let shared_reg = self.reg_status.clone();
                let mut last_status = *shared_reg.lock().unwrap();
                std::thread::spawn(move || loop {
                    if last_status != *shared_reg.lock().unwrap() {
                        last_status = *shared_reg.lock().unwrap();
                        tx.send("+CREG: ".to_owned() + &last_status.to_string());
                    }
                    std::thread::sleep(Duration::from_secs(5))
                });
            }
        }
    }
}
