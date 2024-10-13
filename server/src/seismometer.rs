use crate::pubsub::PubSub;
use crate::record::Record;
use serialport;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

#[derive(Debug)]
pub struct Error {
    pub description: String,
}

impl Error {
    pub fn new(description: String) -> Error {
        Error {
            description: description,
        }
    }
}

impl From<serialport::Error> for Error {
    fn from(serialport_error: serialport::Error) -> Error {
        Error::new(serialport_error.description)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::new(e.to_string())
    }
}

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.description)
//     }
// }

pub struct Seismometer {
    pub_sub: PubSub<Record>,
    port: Box<dyn serialport::SerialPort>,
    buffer: Vec<u8>, // This buffer will be used to store the data read from the serial port
}

impl Seismometer {
    pub fn new(port_name: &str) -> Result<Seismometer, Error> {
        Self::open_port(port_name).and_then(|port| {
            Ok(Seismometer {
                port: port,
                pub_sub: PubSub::<Record>::new(),
                buffer: Vec::with_capacity(4096),
            })
        })
    }

    pub fn subscribe(&self) -> mpsc::Receiver<Record> {
        self.pub_sub.subscribe()
    }

    pub fn run(&mut self, stop: Arc<AtomicBool>) {
        let mut data: [u8; 128] = [0; 128];

        while !stop.load(Ordering::Relaxed) {
            match self.port.read(&mut data) {
                Ok(bytes_read) => {
                    if bytes_read != 0 {
                        self.buffer.extend_from_slice(&data[..bytes_read]);
                        self.process_buffer();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from serial port: {}", e);
                }
            };
        }
    }

    fn process_buffer(&mut self) {
        let newline = self.buffer.iter().position(|&x| x == b'\n');

        if let Some(pos) = newline {
            if let Ok(rec) = Record::from(&self.buffer[..(pos - 1)]) {
                self.pub_sub.publish(rec);
            }

            self.buffer.drain(..pos + 1);
        }
    }

    fn open_port(port_name: &str) -> Result<Box<dyn serialport::SerialPort>, Error> {
        serialport::new(port_name, 500000)
            .timeout(Duration::from_millis(100))
            .open()
            .and_then(|port| {
                // Remove stale data from the buffer
                port.clear(serialport::ClearBuffer::All).unwrap();
                Ok(port)
            })
            .or_else(|e| Err(Error::from(e)))
    }
}
