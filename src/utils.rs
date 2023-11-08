pub mod serial {
    use std::{
        sync::mpsc::{channel, Receiver, Sender},
        time::Duration,
    };

    pub fn read_line_thread(port_name: String, port_rx: Receiver<String>) -> Receiver<String> {
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
                if let Ok(to_send) = port_rx.recv_timeout(Duration::from_millis(2)) {
                    let bytes_written = port.write(to_send.as_bytes()).unwrap();
                }
                if port.read_exact(&mut serial_buf).is_ok() {
                    big_buffer.push(serial_buf[0]);
                    if serial_buf[0] == '\n' as u8 {
                        match std::str::from_utf8(&big_buffer) {
                            Ok(buffer_str) => {
                                if let Some((line, _)) = buffer_str.split_once("\r\n") {
                                    tx.send(line.to_string()).unwrap();
                                    big_buffer.clear();
                                    // return Some(line.into());
                                } else if let Some((line, _)) = buffer_str.split_once('\n') {
                                    tx.send(line.to_string()).unwrap();
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
}
