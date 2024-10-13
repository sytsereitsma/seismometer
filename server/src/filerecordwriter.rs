use crate::record::Record;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

pub struct FileRecordWriter {
    file: std::fs::File,
}

impl FileRecordWriter {
    pub fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("seismodata.txt")?;

        Ok(Self { file })
    }

    pub fn write_record(&mut self, record: &Record, trigger: bool) {
        let res = self.file.write_all(
            format!(
                "{},{},{},{},{},{},{},{},{}\n",
                record.timestamp_utc.timestamp_micros(),
                record.timestamp_us,
                record.x_filt,
                record.y_filt,
                record.z_filt,
                record.x,
                record.y,
                record.z,
                if trigger { "T" } else { "S" }
            )
            .as_bytes(),
        );

        if let Err(e) = res {
            eprintln!("Error writing record: {}", e);
        }
    }
}
