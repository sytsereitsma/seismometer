use record::Record;

use crate::record;

struct ChannelTriggerDetector {
    window: Vec<i32>,
}

impl ChannelTriggerDetector {
    fn is_triggered(&self, reference_index: usize, threshold: i32) -> bool {
        let reference_value = self.window[reference_index];
        let reference_delta =
            self.window[(reference_index + 1) % self.window.len()] - reference_value;

        if reference_delta.abs() < threshold {
            return false;
        }

        let reference_sign = reference_delta.signum();
        let mut triggered = true;

        let start = reference_index + 2;
        for i in start..start + self.window.len() - 2 {
            let value = &self.window[i % self.window.len()];
            let delta = value - reference_value;

            if delta.abs() < threshold || delta.signum() != reference_sign {
                triggered = false;
                break;
            }
        }

        triggered
    }
}

pub struct TriggerDetector {
    threshold: i32,
    x_detector: ChannelTriggerDetector,
    y_detector: ChannelTriggerDetector,
    z_detector: ChannelTriggerDetector,
    index: usize,
    window_full: bool,
}

impl TriggerDetector {
    pub fn new(threshold: i32, window_size: usize) -> TriggerDetector {
        TriggerDetector {
            threshold,
            x_detector: ChannelTriggerDetector {
                window: vec![0; window_size],
            },
            y_detector: ChannelTriggerDetector {
                window: vec![0; window_size],
            },
            z_detector: ChannelTriggerDetector {
                window: vec![0; window_size],
            },
            index: 0,
            window_full: false,
        }
    }

    pub fn detect(&mut self, record: &Record) -> bool {
        self.x_detector.window[self.index] = record.x_filt;
        self.y_detector.window[self.index] = record.y_filt;
        self.z_detector.window[self.index] = record.z_filt;

        let prev_index = self.index;
        self.index = (self.index + 1) % self.x_detector.window.len();

        self.window_full = self.window_full || (prev_index > self.index);

        if !self.window_full {
            return false;
        }

        self.x_detector.is_triggered(self.index, self.threshold)
            || self.y_detector.is_triggered(self.index, self.threshold)
            || self.z_detector.is_triggered(self.index, self.threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_record(x: i32, y: i32, z: i32) -> Record {
        let mut record = Record::default();

        record.x_filt = x;
        record.y_filt = y;
        record.z_filt = z;
        record
    }

    #[test]
    fn test_detect_x_triggers() {
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
        assert!(!detector.detect(&test_record(THRESHOLD + 1, 0, 0))); // [0, THRESHOLD, -]
        assert!(detector.detect(&test_record(THRESHOLD + 1, 0, 0))); // [0, THRESHOLD + 1, THRESHOLD + 1]
    }

    #[test]
    fn test_detect_y_triggers() {
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
        assert!(!detector.detect(&test_record(0, THRESHOLD + 1, 0))); // [0, THRESHOLD, -]
        assert!(detector.detect(&test_record(0, THRESHOLD + 1, 0))); // [0, THRESHOLD + 1, THRESHOLD + 1]
    }

    #[test]
    fn test_detect_z_triggers() {
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
        assert!(!detector.detect(&test_record(0, 0, THRESHOLD + 1))); // [0, THRESHOLD, -]
        assert!(detector.detect(&test_record(0, 0, THRESHOLD + 1))); // [0, THRESHOLD + 1, THRESHOLD + 1]
    }

    #[test]
    fn test_detect_trigger_sign_is_handled() {
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
        assert!(!detector.detect(&test_record(THRESHOLD + 1, 0, 0))); // [0, THRESHOLD, -]
                                                                      // All triggering values should be in the same direction
        assert!(!detector.detect(&test_record(-(THRESHOLD + 10), 0, 0))); // [0, THRESHOLD + 1, -(THRESHOLD + 10)]
    }

    #[test]
    fn test_negative_trigger() {
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
        assert!(!detector.detect(&test_record(-THRESHOLD, 0, 0))); // [0, -THRESHOLD, -]
        assert!(detector.detect(&test_record(-THRESHOLD, 0, 0))); // [0, -THRESHOLD, -THRESHOLD]
    }

    #[test]
    fn test_triggers_are_relative_to_first_value() {
        const THRESHOLD: i32 = 5;
        const FIRST_VALUE: i32 = 123;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(FIRST_VALUE, 0, 0))); // [FIRST_VALUE, -, -]
        assert!(!detector.detect(&test_record(THRESHOLD + FIRST_VALUE + 1, 0, 0))); // [0, THRESHOLD + FIRST_VALUE + 1, -]
        assert!(detector.detect(&test_record(THRESHOLD + FIRST_VALUE + 1, 0, 0)));
        // [0, THRESHOLD + FIRST_VALUE + 1, THRESHOLD + FIRST_VALUE + 1]
    }

    #[test]
    fn test_window_rollover() {
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 3);

        assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
        assert!(!detector.detect(&test_record(1, 0, 0))); // [0, 1, -]
        assert!(!detector.detect(&test_record(2, 0, 0))); // [0, 1, 2]
        assert!(!detector.detect(&test_record(2 + THRESHOLD + 1, 0, 0))); // [2 + THRESHOLD + 1, 1*, 2]
        assert!(detector.detect(&test_record(2 + THRESHOLD + 1, 0, 0))); // [2 + THRESHOLD + 1, 2 + THRESHOLD + 1, 2*]
    }
}
