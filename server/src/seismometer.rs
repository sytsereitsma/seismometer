use crate::pubsub::PubSub;
use crate::record::Record;
use serialport;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::fmt;


// TODO: Make this a proper error type
#[derive(Debug)]
pub struct SeismometerError {
    pub description: String,
}

impl SeismometerError {
    pub fn new(description: &str) -> SeismometerError {
        SeismometerError {
            description: String::from(description),
        }
    }
}

impl From<serialport::Error> for SeismometerError {
    fn from(serialport_error: serialport::Error) -> SeismometerError {
        SeismometerError::new(&serialport_error.description)
    }
}

impl From<std::io::Error> for SeismometerError {
    fn from(e: std::io::Error) -> SeismometerError {
        SeismometerError::new(&e.to_string())
    }
}

impl fmt::Display for SeismometerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

trait Port: Send {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SeismometerError>;    
}

struct PortWrapper {
    port: Box<dyn serialport::SerialPort>,
}

impl Port for PortWrapper {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SeismometerError> {
        self.port.read(buf).or_else(|e| Err(SeismometerError::from(e)))
    }
}

pub struct Seismometer {
    pub_sub: PubSub<Record>,
    port: Box<dyn Port>,
    buffer: Vec<u8>, // This buffer will be used to store the data read from the serial port
    previous_timestamp_us: u32,
    unwrapped_timestamp_us: u64,
}

impl Seismometer {
    pub fn new(port_name: &str) -> Result<Seismometer, SeismometerError> {
        Self::open_port(port_name).and_then(|port| {
            Ok(Seismometer {
                port: port as Box<dyn Port>,
                pub_sub: PubSub::<Record>::new(),
                buffer: Vec::with_capacity(4096),
                previous_timestamp_us: 0,
                unwrapped_timestamp_us: 0,
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
        loop {
            let newline = self.buffer.iter().position(|&x| x == b'\n');

            if let Some(pos) = newline {
                // The pos-1 strips the CR character, Arduino adds a CR LF when sending data using println
                if let Ok(mut rec) = Record::from(&self.buffer[..pos-1]) {
                    let delta = (rec.timestamp_us as u32).wrapping_sub(self.previous_timestamp_us);
                    self.unwrapped_timestamp_us += delta as u64;
                    self.previous_timestamp_us = rec.timestamp_us as u32;
                    rec.timestamp_us = self.unwrapped_timestamp_us;

                    self.pub_sub.publish(rec);
                }

                self.buffer.drain(..pos + 1);
            }
            else {
               break;
            }
        }
    }

    fn open_port(port_name: &str) -> Result<Box<PortWrapper>, SeismometerError> {
        serialport::new(port_name, 500000)
            .timeout(Duration::from_millis(100))
            .open()
            .and_then(|port| {
                // Remove stale data from the buffer
                port.clear(serialport::ClearBuffer::All).unwrap();
                Ok(Box::new(PortWrapper{port}))
            })
            .or_else(|e| Err(SeismometerError::from(e)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestPort {
        data: Arc<Mutex<Vec<u8>>>,
    }

    impl TestPort {
        fn new(data: Arc<Mutex<Vec<u8>>>) -> TestPort {
            TestPort { data: data }
        }
    }

    impl Port for TestPort {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, SeismometerError> {
            let mut data = self.data.lock().unwrap();
            let bytes_read = std::cmp::min(buf.len(), data.len());
            buf[..bytes_read].copy_from_slice(&data[..bytes_read]);
            data.drain(..bytes_read);
            Ok(bytes_read)
        }
    }

    #[test]
    fn test_process_buffer() {
        let data = Arc::new(Mutex::new(Vec::new()));

        let mut seismometer = Seismometer {
            port: Box::new(TestPort::new(data.clone())),
            pub_sub: PubSub::<Record>::new(),
            buffer: Vec::with_capacity(4096),
            previous_timestamp_us: 0,
            unwrapped_timestamp_us: 0,
        };

        let rx = seismometer.subscribe();
        seismometer.buffer.extend_from_slice(b"123456,11,12,13,14,15,16\r\n");
        seismometer.process_buffer();

        let record = rx.recv().unwrap();
        assert_eq!(record.timestamp_us, 123456);
        assert_eq!(record.x_filt, 11);
        assert_eq!(record.y_filt, 12);
        assert_eq!(record.z_filt, 13);
        assert_eq!(record.x, 14);
        assert_eq!(record.y, 15);
        assert_eq!(record.z, 16);
    }

    #[test]
    fn test_process_buffer_with_timestamp_overflow() {
        let data = Arc::new(Mutex::new(Vec::new()));

        let mut seismometer = Seismometer {
            port: Box::new(TestPort::new(data.clone())),
            pub_sub: PubSub::<Record>::new(),
            buffer: Vec::with_capacity(4096),
            previous_timestamp_us: 0,
            unwrapped_timestamp_us: 0,
        };

        let rx = seismometer.subscribe();
        seismometer.buffer.extend_from_slice(b"4294967294,1,1,1,1,1,1\r\n3,1,1,1,1,1,1\r\n15,1,1,1,1,1,1\r\n");
        seismometer.process_buffer();

        let record = rx.recv().unwrap();
        assert_eq!(record.timestamp_us, 4294967294);

        let record = rx.recv().unwrap();
        assert_eq!(record.timestamp_us, 4294967299); // 4294967294 + 5

        let record = rx.recv().unwrap();
        assert_eq!(record.timestamp_us, 4294967311); // 4294967299 + (15 - 3)
    }

}