use crate::record::Record;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

pub trait RecordHandler {
    fn run(&mut self, receiver: mpsc::Receiver<Record>, stop: Arc<AtomicBool>) {
        while !stop.load(Ordering::Relaxed) {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(record) => self.handle(&record),
                Err(_) => (), // Ignore timeouts
            }
        }
    }

    fn handle(&mut self, record: &Record);
}
