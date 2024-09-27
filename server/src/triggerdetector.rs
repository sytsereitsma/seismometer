use record::Record;

use crate::record;


pub struct TriggerDetector {
    threshold: i32,
    window: Vec<Record>,
    index: usize,
    window_full: bool,
}

impl TriggerDetector {
    pub fn new(threshold: i32, window_size: usize) -> TriggerDetector {
        TriggerDetector {
            threshold,
            window: vec![Record::default(); window_size],
            index: 0,
            window_full: false,
        }
    }

    pub fn detect(&mut self, record: &Record) -> bool {
        let prev_index = self.index;
        self.window[self.index] = record.clone();
        self.index = (self.index + 1) % self.window.len();

        if prev_index > self.index {
            self.window_full = true;
        }

        let mut triggered = false;

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut max_z = i32::MIN;

        let end = if self.window_full {
            self.window.len()
        } else {
            self.index
        };

        for i in 0..end {
            let record = &self.window[i];
            min_x = min_x.min(record.x);
            max_x = max_x.max(record.x);
            min_y = min_y.min(record.y);
            max_y = max_y.max(record.y);
            min_z = min_z.min(record.z);
            max_z = max_z.max(record.z);

            if (max_x - min_x) > self.threshold
                || (max_y - min_y) > self.threshold
                || (max_z - min_z) > self.threshold
            {
                triggered = true;
                break;
            }
        }

        triggered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_record(x: i32, y: i32, z: i32) -> Record {
        Record {
            timestamp: 0,
            x,
            y,
            z,
            x_filt: 0,
            y_filt: 0,
            z_filt: 0,
        }
    }

    #[test]
    fn test_detect_x_triggers() {
        let mut detector = TriggerDetector::new(5, 3);
        assert_eq!(detector.detect(&test_record(1, 0, 0)), false); // [1, -, -]
        assert_eq!(detector.detect(&test_record(3, 0, 0)), false); // [1, 3, -]
        assert_eq!(detector.detect(&test_record(-3, 0, 0)), true); // [1, 3, -3]

        // The 3 is now the max value in the window, when it is replaced with 2 it should disable the trigger
        assert_eq!(detector.detect(&test_record(2, 0, 0)), true); // [3, -3, 2]
        assert_eq!(detector.detect(&test_record(2, 0, 0)), false); // [2, -3, 2]
    }

    #[test]
    fn test_detect_y_triggers() {
        let mut detector = TriggerDetector::new(5, 3);
        assert_eq!(detector.detect(&test_record(0, 1, 0)), false); // [1, -, -]
        assert_eq!(detector.detect(&test_record(0, 3, 0)), false); // [1, 3, -]
        assert_eq!(detector.detect(&test_record(0, -3, 0)), true); // [1, 3, -3]

        // The 3 is now the max value in the window, when it is replaced with 2 it should disable the trigger
        assert_eq!(detector.detect(&test_record(0, 2, 0)), true); // [3, -3, 2]
        assert_eq!(detector.detect(&test_record(0, 2, 0)), false); // [2, -3, 2]
    }

    #[test]
    fn test_detect_z_triggers() {
        let mut detector = TriggerDetector::new(5, 3);
        assert_eq!(detector.detect(&test_record(0, 0, 1)), false); // [1, -, -]
        assert_eq!(detector.detect(&test_record(0, 0, 3)), false); // [1, 3, -]
        assert_eq!(detector.detect(&test_record(0, 0, -3)), true); // [1, 3, -3]

        // The 3 is now the max value in the window, when it is replaced with 2 it should disable the trigger
        assert_eq!(detector.detect(&test_record(0, 0, 2)), true); // [3, -3, 2]
        assert_eq!(detector.detect(&test_record(0, 0, 2)), false); // [2, -3, 2]
    }

    #[test]
    fn test_trigger_when_window_is_not_full_yet() {
        // A trigger should be detected when the window is not full yet.
        let mut detector = TriggerDetector::new(5, 3);

        // Pick a high number to make sure we do not get a false trigger based on the window's init values
        assert_eq!(detector.detect(&test_record(99, 0, 0)), false);
        assert_eq!(detector.detect(&test_record(93, 0, 0)), true);
    }
}
