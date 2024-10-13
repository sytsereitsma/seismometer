use crate::record::Record;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use crate::runningrms::RunningRMS;
use std::io::Write;


pub struct RMSRecorder {
    x_rms: RunningRMS,
    y_rms: RunningRMS,
    z_rms: RunningRMS,
    file: std::fs::File,
}

impl RMSRecorder {
    pub fn new(rms_window_size: usize) -> RMSRecorder {
        RMSRecorder {
            x_rms: RunningRMS::new(rms_window_size),
            y_rms: RunningRMS::new(rms_window_size),
            z_rms: RunningRMS::new(rms_window_size),
            file: std::fs::File::create("seismo-rms.txt").unwrap(),
        }
    }

    pub fn run(&mut self, receiver: mpsc::Receiver<Record>, stop: Arc<AtomicBool>) {
        while !stop.load(Ordering::Relaxed) {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(record) => self.handle(&record),
                Err(_) => (), // Ignore timeouts
            }
        }
    }

    pub fn handle(&mut self, record: &Record) {
        if let Some(x_rms) = self.x_rms.add_sample(record.x_filt) {
            if let Some(y_rms) = self.y_rms.add_sample(record.y_filt) {
                if let Some(z_rms) = self.z_rms.add_sample(record.z_filt) {
                    let res = self.file.write_all(
                        format!("{},{},{},{}\n", record.timestamp_us, x_rms, y_rms, z_rms)
                            .as_bytes(),
                    );

                    if let Err(e) = res {
                        eprintln!("Error writing RMS: {}", e);
                    }
                }
            }
        }
    }
}
