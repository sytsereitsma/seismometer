use crate::config::TriggerDetectorConfig;
use crate::emafilter::EMAFilter;
use crate::record::Record;
use std::fs::File;
use std::io::Write;

struct ChannelTriggerDetector {
    filter: EMAFilter,
    delta_window: Vec<i32>,
    delta_index: usize,
    delta_buffer_full: bool,
    threshold: i32,
}

impl ChannelTriggerDetector {
    fn new(
        threshold: i32,
        cutoff_frequency: f64,
        delta_window_size: usize,
    ) -> ChannelTriggerDetector {
        ChannelTriggerDetector {
            filter: EMAFilter::from(1000.0, cutoff_frequency),
            delta_window: vec![0; delta_window_size],
            delta_index: 0,
            delta_buffer_full: false,
            threshold: threshold,
        }
    }

    fn add_sample(&mut self, value: i32) -> bool {
        let filtered_value = self.filter.add_sample(value as f64).round() as i32;

        self.delta_window[self.delta_index] = filtered_value;
        self.delta_index = (self.delta_index + 1) % self.delta_window.len();
        self.delta_buffer_full = self.delta_buffer_full || (self.delta_index == 0);

        if self.delta_buffer_full {
            // Determine min and max in the window
            let mut min: i32 = i32::MAX;
            let mut max: i32 = i32::MIN;

            for v in self.delta_window.iter() {
                let value = *v;

                if value < min {
                    min = value;
                }
                if value > max {
                    max = value;
                }

                if (max - min) > self.threshold {
                    return true;
                }
            }
        }

        false
    }
}

pub struct TriggerDetector {
    x_detector: ChannelTriggerDetector,
    y_detector: ChannelTriggerDetector,
    z_detector: ChannelTriggerDetector,
    file: Option<std::fs::File>,
}

impl TriggerDetector {
    pub fn new(config: &TriggerDetectorConfig) -> TriggerDetector {
        let file = match &config.debug_filename {
            Some(filename) => Some(File::create(filename).unwrap()),
            None => None,
        };

        TriggerDetector {
            x_detector: ChannelTriggerDetector::new(
                config.x_trigger_level,
                config.filter_cutoff_frequency,
                config.delta_window,
            ),
            y_detector: ChannelTriggerDetector::new(
                config.y_trigger_level,
                config.filter_cutoff_frequency,
                config.delta_window,
            ),
            z_detector: ChannelTriggerDetector::new(
                config.z_trigger_level,
                config.filter_cutoff_frequency,
                config.delta_window,
            ),
            file: file,
        }
    }

    pub fn detect(&mut self, record: &Record) -> bool {
        let x_triggered = self.x_detector.add_sample(record.x_filt);
        let y_triggered = self.y_detector.add_sample(record.y_filt);
        let z_triggered = self.z_detector.add_sample(record.z_filt);

        self.write_debug_info(record, x_triggered, y_triggered, z_triggered);

        x_triggered || y_triggered || z_triggered
    }

    fn write_debug_info(&mut self, record: &Record, x_triggered: bool, y_triggered: bool, z_triggered: bool) {
        if let Some(file) = self.file.as_mut() {
            file.write_all(
                format!(
                    "{},{},{},{},{},{},{}\n",
                    record.timestamp_us,
                    self.x_detector.filter.value(),
                    self.y_detector.filter.value(),
                    self.z_detector.filter.value(),
                    x_triggered,
                    y_triggered,
                    z_triggered,
                )
                .as_bytes(),
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_trigger_detector(){
        let mut detector = ChannelTriggerDetector {
            filter: EMAFilter::new(1.0), // Disable the filter to ease testing
            delta_window: vec![0; 3],
            delta_index: 0,
            delta_buffer_full: false,
            threshold: 5,
        };

        assert!(!detector.add_sample(0)); // [0, -, -]
        assert!(!detector.add_sample(1)); // [0, 1, -]
        assert!(!detector.add_sample(2)); // [0, 1, 2]

        // 1 count short of > threshold
        assert!(!detector.add_sample(1 + detector.threshold)); // [Thr + 1, 1, 2]

        // Positive trigger
        assert!(detector.add_sample(3 + detector.threshold)); // [Thr + 1, Thr + 3, 2] 
        
        // And the trigger condition is gone
        assert!(!detector.add_sample(3 + detector.threshold)); // [Thr + 1, Thr + 3, Thr + 3] 
        
        // Negative trigger
        assert!(detector.add_sample(-detector.threshold + 3)); // [Thr + 1, Thr + 3, Thr + 3] 
    }


    fn test_record(x: i32, y: i32, z: i32) -> Record {
        let mut record = Record::default();

        record.x_filt = x;
        record.y_filt = y;
        record.z_filt = z;
        record
    }

    #[test]
    fn test_detect_x_triggers() {
        let config = TriggerDetectorConfig {
            x_trigger_level: 5,
            y_trigger_level: 9999,
            z_trigger_level: 9999,
            filter_cutoff_frequency: 1000.0,
            delta_window: 2,
            debug_filename: None,
        };

        let mut detector = TriggerDetector::new(&config);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -]

        // It should not trigger on y or z
        assert!(!detector.detect(&test_record(0, config.x_trigger_level + 1, config.x_trigger_level + 1))); // [0, 0]

        // It should trigger on x
        assert!(detector.detect(&test_record(config.x_trigger_level + 1, 0, 0))); // [level + 1, 0]
    }

    #[test]
    fn test_detect_y_triggers() {
        let config = TriggerDetectorConfig {
            x_trigger_level: 9999,
            y_trigger_level: 5,
            z_trigger_level: 9999,
            filter_cutoff_frequency: 1000.0,
            delta_window: 2,
            debug_filename: None,
        };

        let mut detector = TriggerDetector::new(&config);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -]

        // It should not trigger on x or z
        assert!(!detector.detect(&test_record(config.y_trigger_level + 1, 0, config.y_trigger_level + 1))); // [0, 0]

        // It should trigger on y
        assert!(detector.detect(&test_record(0, config.y_trigger_level + 1, 0))); // [level + 1, 0]
    }

    #[test]
    fn test_detect_z_triggers() {
        let config = TriggerDetectorConfig {
            x_trigger_level: 9999,
            y_trigger_level: 9999,
            z_trigger_level: 5,
            filter_cutoff_frequency: 1000.0,
            delta_window: 2,
            debug_filename: None,
        };

        let mut detector = TriggerDetector::new(&config);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -]

        // It should not trigger on x or y
        assert!(!detector.detect(&test_record(config.z_trigger_level + 1, config.z_trigger_level + 1, 0))); // [0, 0]

        // It should trigger on z
        assert!(detector.detect(&test_record(0, 0, config.z_trigger_level + 1))); // [level + 1, 0]
    }
}
