use serialport;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use anyhow::Result;

mod record;
mod triggerdetector;
use record::Record;

struct DataRecorder {
    count: usize,
    window: Vec<Record>,
    file: std::fs::File,
    hold_counter: usize,
}

impl DataRecorder {
    fn new(window_size: usize) -> Result<DataRecorder> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("seismodata.txt")?;
    

        Ok(DataRecorder {
            count: 0,
            window: vec![Record::default(); window_size],
            file,
            hold_counter: window_size,
        })
    }

    fn record(&mut self, record: &Record, triggered: bool) {
        if triggered {
            self.hold_counter = 0;
        }

        if self.hold_counter < self.window.len() {
            for i in 0..self.count {
                self.write_record(&self.window[i].clone());
            }

            self.write_record(&record);
            self.count = 0;
            self.hold_counter += 1;
        } else {
            self.window[self.count] = record.clone();
            self.count += 1;

            if self.count == self.window.len() {
                self.count = 0;

                // Calculate average values
                let mut avg = Record::default();
                for r in &self.window {
                    avg.timestamp += r.timestamp;
                    avg.x += r.x;
                    avg.y += r.y;
                    avg.z += r.z;
                    avg.x_filt += r.x_filt;
                    avg.y_filt += r.y_filt;
                    avg.z_filt += r.z_filt;
                }

                avg.timestamp /= self.window.len() as u64;
                avg.x /= self.window.len() as i32;
                avg.y /= self.window.len() as i32;
                avg.z /= self.window.len() as i32;
                avg.x_filt /= self.window.len() as i32;
                avg.y_filt /= self.window.len() as i32;
                avg.z_filt /= self.window.len() as i32; 

                self.write_record(&avg);
            }
        }        
    }

    fn write_record(&mut self, record: &Record) {
        self.file.write_all(format!("{},{},{},{},{},{},{}\n", record.timestamp, record.x, record.y, record.z, record.x_filt, record.y_filt, record.z_filt).as_bytes()).unwrap();
    }
}

fn processing_thread(rx: mpsc::Receiver<Vec<u8>>, stop_thread: Arc<AtomicBool>) {
    const TRIGGER_WINDOW : usize = 100;
    const TRIGGER_LEVEL : i32 = 5000000;

    let mut triggerdetector = triggerdetector::TriggerDetector::new(TRIGGER_LEVEL, TRIGGER_WINDOW);
    let mut data_recorder = DataRecorder::new(TRIGGER_WINDOW).unwrap();

    let mut data: Vec<u8> = Vec::new();

    let mut corrected_timestamp: u64 = 0;
    let mut prev_timestamp: u64 = 0;
    let mut prev_triggered = false;

    loop {
        if stop_thread.load(Ordering::Relaxed) {
            break;
        }

        match rx.recv() {
            Ok(mut chunk) => {
                data.append(&mut chunk);
            }
            Err(_) => {
                continue;
            }
        }

        // let mut data = proc.data.lock().unwrap();
        let newline = data.iter().position(|&x| x == b'\n');

        if let Some(pos) = newline {
            if let Ok(mut rec) = Record::from(&data[..(pos - 1)]) {
                // corrected_timestamp += rec.timestamp - prev_timestamp;
                // prev_timestamp = rec.timestamp;
                // rec.timestamp = corrected_timestamp;

                let triggered = triggerdetector.detect(&rec);
                if triggered != prev_triggered {
                    prev_triggered = triggered;
                    println!("Trigger status changed: {} {}", triggered, rec.timestamp);
                }

                data_recorder.record(&rec, triggered);
            }

            data.drain(..pos + 1);
        }
    }
}

fn main() {
    let stop_thread = Arc::new(AtomicBool::new(false));

    let _ = serialport::new("COM3", 115_200)
        .timeout(Duration::from_millis(1000))
        .open()
        .and_then(|mut port| {
            // Remove stale data from the buffer
            port.clear(serialport::ClearBuffer::All).unwrap();

            let (tx, rx) = mpsc::channel();

            let data_processor_thread = {
                let stop_processor = stop_thread.clone();
                thread::spawn(move || processing_thread(rx, stop_processor))
            };

            let read_thread = {
                let mut buf: Vec<u8> = vec![0; 1024];

                let stop_read = stop_thread.clone();

                thread::spawn(move || loop {
                    if stop_read.load(Ordering::Relaxed) {
                        break;
                    }

                    match port.read(&mut buf) {
                        Ok(n) => {
                            // file.write_all(&buf[..n]).unwrap();
                            //# println!("{}", String::from_utf8((&buf[..n]).to_vec()).unwrap());
                            tx.send(buf[..n].to_vec()).unwrap();
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                })
            };

            Ok((read_thread, data_processor_thread))
        })
        .or_else(|e| {
            println!("Read thread error: {:?}", e);
            Err(e)
        })
        .and_then(|(read_thread, data_processor_thread)| {
            let stop = stop_thread.clone();

            let _ = ctrlc::set_handler(move || {
                stop.store(true, Ordering::Relaxed);
            });

            read_thread.join().unwrap();
            data_processor_thread.join().unwrap();
            println!("Bye");

            Ok(())
        });
}
