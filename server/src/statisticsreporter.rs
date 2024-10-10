use crate::record::Record;
use crate::statistics::Statistics;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

pub struct StatisticsReporter {
    x_stats: Statistics,
    y_stats: Statistics,
    z_stats: Statistics,
}

impl StatisticsReporter {
    pub fn new() -> StatisticsReporter {
        StatisticsReporter {
            x_stats: Statistics::new(),
            y_stats: Statistics::new(),
            z_stats: Statistics::new(),
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
        self.x_stats.add(record.x_filt);
        self.y_stats.add(record.y_filt);
        self.z_stats.add(record.z_filt);

        if self.z_stats.count >= 1000 {
            self.x_stats.print_and_reset("X:");
            self.y_stats.print_and_reset("Y:");
            self.z_stats.print_and_reset("Z:");
        }
    }
}
