use std::collections::VecDeque;

use crate::config::EventRecorderConfig;
use crate::filerecordwriter::FileRecordWriter;
use crate::record::Record;
use crate::recordhandler::RecordHandler;
use crate::triggerdetector::TriggerDetector;

pub trait Detector {
    fn detect(&mut self, record: &Record) -> bool;
}

impl Detector for TriggerDetector {
    fn detect(&mut self, record: &Record) -> bool {
        TriggerDetector::detect(self, record)
    }
}

pub trait Writer {
    fn write_record(&mut self, record: &Record, trigger: bool);
}

impl Writer for FileRecordWriter {
    fn write_record(&mut self, record: &Record, trigger: bool) {
        FileRecordWriter::write_record(self, record, trigger);
    }
}

pub struct EventRecorder {
    triggerdetector: Box<dyn Detector + Send>,
    data_writer: Box<dyn Writer + Send>,
    /// Time window of data to log before the trigger event
    pre_trigger_time_us: u32,
    /// Time window of data to log after the trigger event
    post_trigger_time_us: u32,
    /// Last trigger event time in microseconds
    last_trigger_time_us: u32,
    pre_trigger_buffer: VecDeque<Record>,
    triggered: bool,
}

impl EventRecorder {
    pub fn new(config: &EventRecorderConfig) -> EventRecorder {
        EventRecorder {
            triggerdetector: Box::new(TriggerDetector::new(
                config.x_trigger_level,
                config.y_trigger_level,
                config.z_trigger_level,
                config.filter_cutoff_frequency,
                config.delta_window,
            )),
            data_writer: Box::new(FileRecordWriter::new(&config.filename).unwrap()),
            pre_trigger_time_us: config.pre_trigger_time_ms as u32 * 1000,
            post_trigger_time_us: config.post_trigger_time_ms as u32 * 1000,
            pre_trigger_buffer: VecDeque::with_capacity(128),
            last_trigger_time_us: 0,
            triggered: false,
        }
    }

    fn check_trigger_status(&mut self, record: &Record) {
        let triggered = self.triggerdetector.detect(&record);

        if triggered {
            self.last_trigger_time_us = record.timestamp_us;
            self.triggered = true;
        } else if self.triggered {
            let delta_us = record.timestamp_us - self.last_trigger_time_us;
            self.triggered = delta_us <= self.post_trigger_time_us;
        }
    }

    fn handle(&mut self, record: &Record) {
        let prev_triggered = self.triggered;
        self.check_trigger_status(record);
        let new_event = self.triggered && prev_triggered != self.triggered;

        if prev_triggered != self.triggered {
            println!("Trigger status changed: {}", self.triggered);
        }

        self.clean_pre_trigger_buffer(record.timestamp_us);

        if new_event {
            for r in self.pre_trigger_buffer.iter() {
                self.data_writer.write_record(r, false);
            }
            self.pre_trigger_buffer.clear();
        }

        if !self.triggered {
            self.pre_trigger_buffer.push_back(record.clone());
        } else {
            self.data_writer.write_record(record, new_event);
        }
    }

    fn clean_pre_trigger_buffer(&mut self, timestamp_us: u32) {
        let mut end_index: usize = 0;

        while end_index < self.pre_trigger_buffer.len()
            && (timestamp_us.wrapping_sub(self.pre_trigger_buffer[end_index].timestamp_us))
                > self.pre_trigger_time_us
        {
            end_index += 1;
        }

        self.pre_trigger_buffer.drain(0..end_index);
    }
}

impl RecordHandler for EventRecorder {
    fn handle(&mut self, record: &Record) {
        Self::handle(self, record);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    struct TestDetector {
        triggered: Arc<AtomicBool>,
    }

    impl Detector for TestDetector {
        fn detect(&mut self, _: &Record) -> bool {
            self.triggered.load(Ordering::Relaxed)
        }
    }
    struct TestWriter {
        timestamps: Arc<Mutex<Vec<u32>>>,
    }

    impl Writer for TestWriter {
        fn write_record(&mut self, record: &Record, _trigger: bool) {
            self.timestamps.lock().unwrap().push(record.timestamp_us);
        }
    }

    struct TestContext {
        triggered: Arc<AtomicBool>,
        timestamps: Arc<Mutex<Vec<u32>>>,
        recorder: EventRecorder,
        record: Record,
    }

    impl TestContext {
        fn new() -> TestContext {
            // These need to be synched, because of the Send trait requirement, even though we are not using threads in the test
            let triggered = Arc::new(AtomicBool::new(false));
            let timestamps = Arc::new(Mutex::new(Vec::new()));

            let recorder = EventRecorder {
                triggerdetector: Box::new(TestDetector {
                    triggered: triggered.clone(),
                }),
                data_writer: Box::new(TestWriter {
                    timestamps: timestamps.clone(),
                }),
                pre_trigger_time_us: 1250,
                post_trigger_time_us: 1500,
                pre_trigger_buffer: VecDeque::with_capacity(128),
                last_trigger_time_us: 0,
                triggered: false,
            };

            TestContext {
                triggered,
                timestamps,
                recorder,
                record: Record::default(),
            }
        }
    }

    #[test]
    fn check_trigger_status() {
        let mut ctx = TestContext::new();

        // First check a false trigger
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, false);

        // Next a true trigger
        ctx.record.timestamp_us = 2000;
        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, true);

        // The trigger should remain active for the next 1500 us, even when the source is gone
        ctx.triggered.store(false, Ordering::Relaxed);
        ctx.record.timestamp_us += ctx.recorder.post_trigger_time_us;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, true);

        // But it should reset after the post trigger time
        ctx.record.timestamp_us += 1;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, false);
    }

    #[test]
    fn check_trigger_status_retrigger_in_post_trigger_time() {
        let mut ctx = TestContext::new();

        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.record.timestamp_us = 12345;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, true);

        // The trigger should remain active for the next 1500 us, even when the source is gone
        ctx.triggered.store(false, Ordering::Relaxed);
        ctx.record.timestamp_us += ctx.recorder.post_trigger_time_us;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, true);

        // Now we retrigger on the verge of the post trigger time
        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.record.timestamp_us += 1;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, true);

        // For good measure check the trigger ends after the post trigger time
        ctx.triggered.store(false, Ordering::Relaxed);

        // First just before it should  reset
        ctx.record.timestamp_us += ctx.recorder.post_trigger_time_us;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, true);

        // Then after it should reset
        ctx.record.timestamp_us += 1;
        ctx.recorder.check_trigger_status(&ctx.record);
        assert_eq!(ctx.recorder.triggered, false);
    }

    #[test]
    fn pre_trigger_buffer() {
        let mut ctx = TestContext::new();
        let first_timestamp = 12345;

        // One sample in the buffer (this should be dropped eventually)
        ctx.record.timestamp_us = first_timestamp;
        ctx.recorder.handle(&ctx.record);

        // Second sample in the buffer (this should be dropped eventually)
        ctx.record.timestamp_us += 1;
        ctx.recorder.handle(&ctx.record);

        // This sample should pop the first sample, because it is too old
        ctx.record.timestamp_us = first_timestamp + ctx.recorder.pre_trigger_time_us + 1;
        ctx.recorder.handle(&ctx.record);

        // Now trigger the event, this should pop the 2nd sample and write the buffer
        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.record.timestamp_us += 1;
        ctx.recorder.handle(&ctx.record);
        assert!(ctx.recorder.pre_trigger_buffer.is_empty());

        // Add one more sample to make sure the pre-trigger buffer is not written too
        ctx.record.timestamp_us += 1;
        ctx.recorder.handle(&ctx.record);

        // All pre trigger buffer samples should be written (including the trigger samples)
        let timestamps = ctx.timestamps.lock().unwrap();
        assert_eq!(timestamps.len(), 3);
        assert_eq!(
            timestamps[0],
            first_timestamp + ctx.recorder.pre_trigger_time_us + 1
        );
        assert_eq!(
            timestamps[1],
            first_timestamp + ctx.recorder.pre_trigger_time_us + 2
        );
        assert_eq!(
            timestamps[2],
            first_timestamp + ctx.recorder.pre_trigger_time_us + 3
        );
    }

    #[test]
    fn post_trigger_samples() {
        let mut ctx = TestContext::new();
        let first_timestamp = 12345;

        // Trigger a single sample
        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.record.timestamp_us = first_timestamp;
        ctx.recorder.handle(&ctx.record);

        ctx.triggered.store(false, Ordering::Relaxed);
        ctx.record.timestamp_us += ctx.recorder.post_trigger_time_us;
        ctx.recorder.handle(&ctx.record);

        // This sample should not be logged
        ctx.record.timestamp_us += 1;
        ctx.recorder.handle(&ctx.record);

        // All pre trigger buffer samples should be written (including the trigger samples)
        let timestamps = ctx.timestamps.lock().unwrap();
        assert_eq!(timestamps.len(), 2);
        assert_eq!(timestamps[0], first_timestamp);
        assert_eq!(
            timestamps[1],
            first_timestamp + ctx.recorder.post_trigger_time_us
        );
    }

    #[test]
    fn post_trigger_with_retrigger() {
        let mut ctx = TestContext::new();
        let first_timestamp = 12345;

        // Trigger a single sample
        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.record.timestamp_us = first_timestamp;
        ctx.recorder.handle(&ctx.record);

        ctx.triggered.store(false, Ordering::Relaxed);
        ctx.record.timestamp_us += ctx.recorder.post_trigger_time_us;
        ctx.recorder.handle(&ctx.record);

        // Retrigger
        ctx.triggered.store(true, Ordering::Relaxed);
        ctx.record.timestamp_us += 1;
        ctx.recorder.handle(&ctx.record);

        ctx.triggered.store(false, Ordering::Relaxed);
        ctx.record.timestamp_us += ctx.recorder.post_trigger_time_us;
        ctx.recorder.handle(&ctx.record);

        // This sample should not be logged
        ctx.record.timestamp_us += 1;
        ctx.recorder.handle(&ctx.record);

        // All pre trigger buffer samples should be written (including the trigger samples)
        let timestamps = ctx.timestamps.lock().unwrap();
        assert_eq!(timestamps.len(), 4);
        assert_eq!(timestamps[0], first_timestamp);
        assert_eq!(
            timestamps[1],
            first_timestamp + ctx.recorder.post_trigger_time_us
        );
        assert_eq!(
            timestamps[2],
            first_timestamp + ctx.recorder.post_trigger_time_us + 1
        );
        assert_eq!(
            timestamps[3],
            first_timestamp + ctx.recorder.post_trigger_time_us * 2 + 1
        );
    }
}
