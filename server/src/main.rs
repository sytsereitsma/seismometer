use serialport;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

mod filerecordwriter;
mod record;
mod triggerdetector;

use filerecordwriter::FileRecordWriter;
use record::Record;

fn processing_thread(rx: mpsc::Receiver<Vec<u8>>, stop_thread: Arc<AtomicBool>) {
    const TRIGGER_WINDOW: usize = 100;
    const AVERAGING_WINDOW: usize = 100;
    const TRIGGER_LEVEL: i32 = 500000;

    let mut triggerdetector = triggerdetector::TriggerDetector::new(TRIGGER_LEVEL, TRIGGER_WINDOW);
    let mut data_writer = FileRecordWriter::new(AVERAGING_WINDOW).unwrap();

    let mut data: Vec<u8> = Vec::new();
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
            if let Ok(rec) = Record::from(&data[..(pos - 1)]) {
                let triggered = triggerdetector.detect(&rec);
                if triggered != prev_triggered {
                    prev_triggered = triggered;
                    println!("Trigger status changed: {} {}", triggered, rec.timestamp_us);
                }

                data_writer.record(&rec, triggered);
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
