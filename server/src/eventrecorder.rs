use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use toml::value::Time;

use crate::config::Config;
use crate::filerecordwriter::FileRecordWriter;
use crate::record::Record;
use crate::triggerdetector::TriggerDetector;

pub struct EventRecorder {
    triggerdetector: TriggerDetector,
    data_writer: FileRecordWriter,
    prev_triggered: bool,
}

impl EventRecorder {
    pub fn new(config: &Config) -> EventRecorder {
        EventRecorder {
            triggerdetector: TriggerDetector::new(config.trigger_level, config.trigger_window),
            data_writer: FileRecordWriter::new(config.averaging_window).unwrap(),
            prev_triggered: false,
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

    fn handle(&mut self, record: &Record) {
        let triggered = self.triggerdetector.detect(&record);

        if triggered != self.prev_triggered {
            self.prev_triggered = triggered;

            println!(
                "Trigger status changed: {} {}",
                triggered, record.timestamp_us
            );
        }

        self.data_writer.record(&record, triggered);
    }
}
