use crate::record::Record;
use anyhow::Result;
use chrono::DateTime;
use std::fs::OpenOptions;
use std::io::Write;

pub struct FileRecordWriter {
    count: usize,
    window: Vec<Record>,
    file: std::fs::File,
    hold_counter: usize,
}

impl FileRecordWriter {
    pub fn new(window_size: usize) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("seismodata.txt")?;

        Ok(Self {
            count: 0,
            window: vec![Record::default(); window_size],
            file,
            hold_counter: window_size,
        })
    }

    pub fn record(&mut self, record: &Record, triggered: bool) {
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
                let mut utc_sum: i64 = 0;

                for r in &self.window {
                    avg.timestamp_us += r.timestamp_us;
                    utc_sum += r.timestamp_utc.timestamp_micros();
                    avg.x += r.x;
                    avg.y += r.y;
                    avg.z += r.z;
                    avg.x_filt += r.x_filt;
                    avg.y_filt += r.y_filt;
                    avg.z_filt += r.z_filt;
                }

                avg.timestamp_us /= self.window.len() as u64;
                avg.timestamp_utc =
                    DateTime::from_timestamp_micros((utc_sum / self.window.len() as i64) as i64)
                        .unwrap();
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
        self.file
            .write_all(
                format!(
                    "{},{},{},{},{},{},{},{}\n",
                    record.timestamp_utc.timestamp_micros(),
                    record.timestamp_us,
                    record.x_filt,
                    record.y_filt,
                    record.z_filt,
                    record.x,
                    record.y,
                    record.z
                )
                .as_bytes(),
            )
            .unwrap();
    }
}
